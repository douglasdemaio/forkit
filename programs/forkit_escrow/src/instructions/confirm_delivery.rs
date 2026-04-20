use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::hash;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::ForkitError;
use crate::instructions::create_order::LOYALTY_POINTS_BPS;

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
        seeds = [ESCROW_VAULT_SEED, &order.order_id.to_le_bytes()],
        bump,
    )]
    pub escrow_vault: Account<'info, TokenAccount>,

    #[account(
        seeds = [ProtocolConfig::SEED],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,

    /// Restaurant's token account — must hold the correct mint
    #[account(
        mut,
        constraint = restaurant_token_account.owner == order.restaurant
            @ ForkitError::Unauthorized,
        constraint = restaurant_token_account.mint == order.token_mint
            @ ForkitError::UnsupportedMint,
    )]
    pub restaurant_token_account: Account<'info, TokenAccount>,

    /// Driver's token account — must hold the correct mint
    #[account(
        mut,
        constraint = driver_token_account.owner == order.driver
            @ ForkitError::Unauthorized,
        constraint = driver_token_account.mint == order.token_mint
            @ ForkitError::UnsupportedMint,
    )]
    pub driver_token_account: Account<'info, TokenAccount>,

    /// Treasury token account for protocol fee — must hold the correct mint
    #[account(
        mut,
        constraint = treasury_token_account.owner == protocol_config.treasury_wallet
            @ ForkitError::Unauthorized,
        constraint = treasury_token_account.mint == order.token_mint
            @ ForkitError::UnsupportedMint,
    )]
    pub treasury_token_account: Account<'info, TokenAccount>,

    pub customer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

/// Confirms delivery, pays restaurant + driver + treasury.
/// If the escrow was overfunded (multiple contributors), excess remains in
/// the vault for contributors to claim via `claim_deposit` (reimbursement).
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

    // Use u128 for intermediate fee calculations to prevent overflow on large orders.
    let food_amt = order.food_amount as u128;
    let delivery_amt = order.delivery_amount as u128;
    let total = food_amt
        .checked_add(delivery_amt)
        .ok_or(ForkitError::ArithmeticOverflow)?;

    let protocol_fee = order.protocol_fee as u128;

    // food_fee = protocol_fee * food_amount / total  (rounded down)
    let food_fee = protocol_fee
        .checked_mul(food_amt)
        .ok_or(ForkitError::ArithmeticOverflow)?
        .checked_div(total)
        .ok_or(ForkitError::ArithmeticOverflow)? as u64;

    // delivery_fee = protocol_fee - food_fee  (takes up any rounding remainder)
    let delivery_fee = order.protocol_fee
        .checked_sub(food_fee)
        .ok_or(ForkitError::ArithmeticOverflow)?;

    let restaurant_payout = order.food_amount
        .checked_sub(food_fee)
        .ok_or(ForkitError::ArithmeticOverflow)?;
    let driver_payout = order.delivery_amount
        .checked_sub(delivery_fee)
        .ok_or(ForkitError::ArithmeticOverflow)?;

    // Sanity: total outflow == escrow_target (food + delivery)
    // restaurant_payout + driver_payout + protocol_fee
    //   = (food - food_fee) + (delivery - delivery_fee) + (food_fee + delivery_fee)
    //   = food + delivery  ✓

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

    // Loyalty points: 1% of total order value. Backend listens for this event
    // and calls earn_points on the loyalty program with this value.
    let loyalty_points = (order.food_amount as u128 + order.delivery_amount as u128)
        .checked_mul(LOYALTY_POINTS_BPS as u128)
        .unwrap_or(0)
        .checked_div(10_000)
        .unwrap_or(0) as u64;
    let is_ai_order = order.ai_confidence > 0;

    order.status = OrderStatus::Settled;
    order.delivery_confirmed_at = clock.unix_timestamp;

    emit!(DeliveryConfirmed {
        order_id: order.order_id,
        timestamp: clock.unix_timestamp,
        restaurant_payout,
        driver_payout,
        protocol_fee: order.protocol_fee,
        loyalty_points_for_customer: loyalty_points,
        is_ai_order,
    });

    Ok(())
}
