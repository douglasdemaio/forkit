use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::ForkitError;

#[derive(Accounts)]
pub struct TimeoutRefund<'info> {
    #[account(
        mut,
        seeds = [Order::SEED, &order.order_id.to_le_bytes()],
        bump = order.bump,
    )]
    pub order: Account<'info, Order>,

    /// Permissionless crank — anyone can call
    pub cranker: Signer<'info>,
}

/// Marks an order as Refunded if it has timed out.
/// Individual contributors then claim refunds via `refund_contributor`.
pub fn handler(ctx: Context<TimeoutRefund>) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let clock = Clock::get()?;

    let timed_out = match order.status {
        // Funding timeout — order never got fully funded
        OrderStatus::Created => {
            clock.unix_timestamp > order.created_at + FUNDING_TIMEOUT_SECONDS
        }
        // Prep timeout
        OrderStatus::Funded | OrderStatus::Preparing => {
            clock.unix_timestamp > order.created_at + PREP_TIMEOUT_SECONDS
        }
        // Pickup timeout
        OrderStatus::ReadyForPickup => {
            clock.unix_timestamp > order.created_at + PREP_TIMEOUT_SECONDS + PICKUP_TIMEOUT_SECONDS
        }
        _ => false,
    };

    require!(timed_out, ForkitError::NotTimedOut);

    order.status = OrderStatus::Refunded;

    emit!(OrderRefunded {
        order_id: order.order_id,
        reason: "Timeout".to_string(),
    });

    Ok(())
}
