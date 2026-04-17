use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::ForkitError;

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    #[account(
        mut,
        seeds = [Order::SEED, &order.order_id.to_le_bytes()],
        bump = order.bump,
        has_one = customer @ ForkitError::Unauthorized,
    )]
    pub order: Account<'info, Order>,

    pub customer: Signer<'info>,
}

/// Cancel sets the order to Cancelled. Individual contributors are refunded
/// via the permissionless `refund_contributor` instruction.
/// This avoids the need to pass all contributor accounts in one transaction.
pub fn handler(ctx: Context<CancelOrder>) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let clock = Clock::get()?;

    require!(
        order.status == OrderStatus::Created || order.status == OrderStatus::Funded,
        ForkitError::InvalidOrderStatus
    );
    require!(
        clock.unix_timestamp < order.cancel_deadline,
        ForkitError::CancelWindowClosed
    );

    order.status = OrderStatus::Cancelled;

    emit!(OrderCancelled {
        order_id: order.order_id,
    });

    Ok(())
}
