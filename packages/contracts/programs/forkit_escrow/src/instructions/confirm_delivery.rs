use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::hash;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::ForkitError;

#[derive(Accounts)]
pub struct ConfirmDelivery<'info> {
    #[account(
        mut,
        seeds = [Order::SEED, &order.order_id.to_le_bytes()],
        bump = order.bump,
        has_one = customer @ ForkitError::Unauthorized,
    )]
    pub order: Account<'info, Order>,

    #[account(
        mut,
        seeds = [b"escrow_vault", &order.order_id.to_le_bytes()],
        bump,
    )]
    pub escrow_vault: Account<'info, TokenAccount>,

    #[account(
        seeds = [ProtocolConfig::SEED],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,

    /// Restaurant's token account to receive food payment
    #[account(
        mut,
        constraint = restaurant_token_account.owner == order.restaurant,
    )]
    pub restaurant_token_account: Account<'info, TokenAccount>,

    /// Driver's token account to receive delivery payment
    #[account(
        mut,
        constraint = driver_token_account.owner == order.driver,
    )]
    pub driver_token_account: Account<'info, TokenAccount>,

    /// Treasury token account for protocol fee
    #[account(
        mut,
        constraint = treasury_token_account.owner == protocol_config.treasury_wallet,
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,

    /// Customer's token account to receive deposit refund
    #[account(
        mut,
        constraint = customer_token_account.owner == customer.key(),
    )]
    pub customer_token_account: Account<'info, TokenAccount>,

    pub customer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<ConfirmDelivery>, code_b: String) -> Result<()> {
    let order = &mut ctx.accounts.order;

    require!(
        order.status == OrderStatus::PickedUp,
        ForkitError::InvalidOrderStatus
    );

    // Verify CODE_B
    let code_hash = hash(code_b.as_bytes());
    require!(
        code_hash.to_bytes() == order.code_b_hash,
        ForkitError::InvalidCode
    );

    let clock = Clock::get()?;
    let order_id_bytes = order.order_id.to_le_bytes();
    let seeds = &[b"escrow_vault" as &[u8], &order_id_bytes, &[ctx.bumps.escrow_vault]];

    // Calculate fee split proportional to food/delivery amounts
    let total = order.food_amount + order.delivery_amount;
    let food_fee = order
        .protocol_fee
        .checked_mul(order.food_amount)
        .ok_or(ForkitError::ArithmeticOverflow)?
        .checked_div(total)
        .ok_or(ForkitError::ArithmeticOverflow)?;
    let delivery_fee = order.protocol_fee.checked_sub(food_fee)
        .ok_or(ForkitError::ArithmeticOverflow)?;

    let restaurant_payout = order.food_amount.checked_sub(food_fee)
        .ok_or(ForkitError::ArithmeticOverflow)?;
    let driver_payout = order.delivery_amount.checked_sub(delivery_fee)
        .ok_or(ForkitError::ArithmeticOverflow)?;

    // Transfer to restaurant
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_vault.to_account_info(),
                to: ctx.accounts.restaurant_token_account.to_account_info(),
                authority: ctx.accounts.escrow_vault.to_account_info(),
            },
            &[seeds],
        ),
        restaurant_payout,
    )?;

    // Transfer to driver
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_vault.to_account_info(),
                to: ctx.accounts.driver_token_account.to_account_info(),
                authority: ctx.accounts.escrow_vault.to_account_info(),
            },
            &[seeds],
        ),
        driver_payout,
    )?;

    // Transfer protocol fee to treasury
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_vault.to_account_info(),
                to: ctx.accounts.treasury_token_account.to_account_info(),
                authority: ctx.accounts.escrow_vault.to_account_info(),
            },
            &[seeds],
        ),
        order.protocol_fee,
    )?;

    // Return deposit to customer
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_vault.to_account_info(),
                to: ctx.accounts.customer_token_account.to_account_info(),
                authority: ctx.accounts.escrow_vault.to_account_info(),
            },
            &[seeds],
        ),
        order.deposit_amount,
    )?;

    order.status = OrderStatus::Settled;
    order.delivery_confirmed_at = clock.unix_timestamp;

    emit!(DeliveryConfirmed {
        order_id: order.order_id,
        timestamp: clock.unix_timestamp,
        restaurant_payout,
        driver_payout,
        protocol_fee: order.protocol_fee,
    });

    Ok(())
}
