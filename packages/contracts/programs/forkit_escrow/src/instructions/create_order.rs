use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use crate::state::*;
use crate::errors::ForkitError;

#[derive(Accounts)]
#[instruction(order_id: u64)]
pub struct CreateOrder<'info> {
    #[account(
        init,
        payer = customer,
        space = Order::SPACE,
        seeds = [Order::SEED, &order_id.to_le_bytes()],
        bump,
    )]
    pub order: Account<'info, Order>,

    #[account(
        seeds = [ProtocolConfig::SEED],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,

    /// The restaurant wallet that will receive food payment
    /// CHECK: Validated by the caller; restaurant identity is stored on the order
    pub restaurant: UncheckedAccount<'info>,

    pub token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = customer,
        token::mint = token_mint,
        token::authority = escrow_vault,
        seeds = [b"escrow_vault", &order_id.to_le_bytes()],
        bump,
    )]
    pub escrow_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = customer_token_account.owner == customer.key(),
        constraint = customer_token_account.mint == token_mint.key(),
    )]
    pub customer_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub customer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<CreateOrder>,
    order_id: u64,
    food_amount: u64,
    delivery_amount: u64,
    code_a_hash: [u8; 32],
    code_b_hash: [u8; 32],
) -> Result<()> {
    let config = &ctx.accounts.protocol_config;
    let mint_key = ctx.accounts.token_mint.key();

    // Validate mint is accepted
    require!(config.is_mint_accepted(&mint_key), ForkitError::UnsupportedMint);

    // Calculate amounts
    let total = food_amount
        .checked_add(delivery_amount)
        .ok_or(ForkitError::ArithmeticOverflow)?;
    let deposit_amount = total
        .checked_mul(DEPOSIT_MULTIPLIER)
        .ok_or(ForkitError::ArithmeticOverflow)?;
    let protocol_fee = total
        .checked_mul(config.fee_basis_points as u64)
        .ok_or(ForkitError::ArithmeticOverflow)?
        .checked_div(10000)
        .ok_or(ForkitError::ArithmeticOverflow)?;

    // Total to transfer into escrow: food + delivery + deposit
    let escrow_total = total
        .checked_add(deposit_amount)
        .ok_or(ForkitError::ArithmeticOverflow)?;

    // Transfer funds to escrow vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.customer_token_account.to_account_info(),
                to: ctx.accounts.escrow_vault.to_account_info(),
                authority: ctx.accounts.customer.to_account_info(),
            },
        ),
        escrow_total,
    )?;

    let clock = Clock::get()?;

    // Initialize order
    let order = &mut ctx.accounts.order;
    order.order_id = order_id;
    order.customer = ctx.accounts.customer.key();
    order.restaurant = ctx.accounts.restaurant.key();
    order.driver = Pubkey::default();
    order.token_mint = mint_key;
    order.food_amount = food_amount;
    order.delivery_amount = delivery_amount;
    order.deposit_amount = deposit_amount;
    order.protocol_fee = protocol_fee;
    order.code_a_hash = code_a_hash;
    order.code_b_hash = code_b_hash;
    order.status = OrderStatus::Created;
    order.created_at = clock.unix_timestamp;
    order.cancel_deadline = clock.unix_timestamp + CANCEL_WINDOW_SECONDS;
    order.pickup_confirmed_at = 0;
    order.delivery_confirmed_at = 0;
    order.bump = ctx.bumps.order;

    emit!(OrderCreated {
        order_id,
        customer: order.customer,
        restaurant: order.restaurant,
        token_mint: mint_key,
        food_amount,
        delivery_amount,
        deposit_amount,
        protocol_fee,
    });

    Ok(())
}
