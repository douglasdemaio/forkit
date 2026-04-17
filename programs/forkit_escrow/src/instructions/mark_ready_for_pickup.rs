use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::ForkitError;

#[derive(Accounts)]
pub struct MarkReadyForPickup<'info> {
    #[account(
        mut,
        seeds = [Order::SEED, &order.order_id.to_le_bytes()],
        bump = order.bump,
        has_one = restaurant @ ForkitError::Unauthorized,
    )]
    pub order: Account<'info, Order>,
    pub restaurant: Signer<'info>,
}

pub fn handler(ctx: Context<MarkReadyForPickup>) -> Result<()> {
    let order = &mut ctx.accounts.order;

    require!(
        order.status == OrderStatus::Preparing,
        ForkitError::InvalidOrderStatus
    );

    order.status = OrderStatus::ReadyForPickup;

    emit!(OrderReadyForPickup {
        order_id: order.order_id,
    });

    Ok(())
}
