use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::ForkitError;

#[derive(Accounts)]
pub struct OpenDispute<'info> {
    #[account(
        mut,
        seeds = [Order::SEED, &order.order_id.to_le_bytes()],
        bump = order.bump,
        has_one = customer @ ForkitError::Unauthorized,
    )]
    pub order: Account<'info, Order>,
    pub customer: Signer<'info>,
}

pub fn handler(ctx: Context<OpenDispute>) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let clock = Clock::get()?;

    require!(
        order.status == OrderStatus::PickedUp,
        ForkitError::InvalidOrderStatus
    );

    // Can only open dispute after delivery timeout
    require!(
        clock.unix_timestamp > order.pickup_confirmed_at + DELIVERY_TIMEOUT_SECONDS,
        ForkitError::DeliveryTimeoutNotReached
    );

    order.status = OrderStatus::Disputed;

    emit!(DisputeOpened {
        order_id: order.order_id,
        customer: order.customer,
    });

    Ok(())
}
