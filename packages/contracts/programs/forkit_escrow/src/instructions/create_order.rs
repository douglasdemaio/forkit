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

    /// The creator's contribution record (may be 0 if they don't fund yet)
    #[account(
        init,
        payer = customer,
        space = Contribution::SPACE,
        seeds = [Contribution::SEED, &order_id.to_le_bytes(), customer.key().as_ref()],
        bump,
    )]
    pub contribution: Account<'info, Contribution>,

    #[account(
        seeds = [ProtocolConfig::SEED],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,

    /// CHECK: Validated by the caller; restaurant identity stored on the order
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
    initial_contribution: u64,
    code_a_hash: [u8; 32],
    code_b_hash: [u8; 32],
) -> Result<()> {
    let config = &ctx.accounts.protocol_config;
    let mint_key = ctx.accounts.token_mint.key();

    require!(config.is_mint_accepted(&mint_key), ForkitError::UnsupportedMint);

    // Calculate amounts
    let total = food_amount
        .checked_add(delivery_amount)
        .ok_or(ForkitError::ArithmeticOverflow)?;
    let deposit_amount = total
        .checked_mul(DEPOSIT_BASIS_POINTS)
        .ok_or(ForkitError::ArithmeticOverflow)?
        .checked_div(10000)
        .ok_or(ForkitError::ArithmeticOverflow)?;
    let protocol_fee = total
        .checked_mul(config.fee_basis_points as u64)
        .ok_or(ForkitError::ArithmeticOverflow)?
        .checked_div(10000)
        .ok_or(ForkitError::ArithmeticOverflow)?;

    // Total needed in escrow: food + delivery + deposit
    let escrow_target = total
        .checked_add(deposit_amount)
        .ok_or(ForkitError::ArithmeticOverflow)?;

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
    order.escrow_target = escrow_target;
    order.escrow_funded = 0;
    order.contributor_count = 0;
    order.code_a_hash = code_a_hash;
    order.code_b_hash = code_b_hash;
    order.status = OrderStatus::Created;
    order.created_at = clock.unix_timestamp;
    order.cancel_deadline = clock.unix_timestamp + CANCEL_WINDOW_SECONDS;
    order.pickup_confirmed_at = 0;
    order.delivery_confirmed_at = 0;
    order.bump = ctx.bumps.order;

    // Initialize contribution record for the creator
    let contribution = &mut ctx.accounts.contribution;
    contribution.order_id = order_id;
    contribution.contributor = ctx.accounts.customer.key();
    contribution.amount = 0;
    contribution.bump = ctx.bumps.contribution;

    // If initial contribution > 0, fund immediately
    if initial_contribution > 0 {
        // Cap at escrow target
        let actual_contribution = initial_contribution.min(escrow_target);

        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.customer_token_account.to_account_info(),
                    to: ctx.accounts.escrow_vault.to_account_info(),
                    authority: ctx.accounts.customer.to_account_info(),
                },
            ),
            actual_contribution,
        )?;

        contribution.amount = actual_contribution;
        order.escrow_funded = actual_contribution;
        order.contributor_count = 1;

        // Check if fully funded in one go
        if order.is_fully_funded() {
            order.status = OrderStatus::Funded;
            emit!(OrderFunded {
                order_id,
                total_funded: order.escrow_funded,
                contributor_count: order.contributor_count,
            });
        }

        emit!(ContributionMade {
            order_id,
            contributor: ctx.accounts.customer.key(),
            amount: actual_contribution,
            total_funded: order.escrow_funded,
            escrow_target,
            fully_funded: order.is_fully_funded(),
        });
    }

    emit!(OrderCreated {
        order_id,
        customer: order.customer,
        restaurant: order.restaurant,
        token_mint: mint_key,
        food_amount,
        delivery_amount,
        deposit_amount,
        escrow_target,
        protocol_fee,
    });

    Ok(())
}
