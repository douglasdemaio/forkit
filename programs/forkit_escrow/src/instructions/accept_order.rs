use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::ForkitError;

#[derive(Accounts)]
pub struct AcceptOrder<'info> {
    #[account(
        mut,
        seeds = [Order::SEED, &order.order_id.to_le_bytes()],
        bump = order.bump,
    )]
    pub order: Account<'info, Order>,
    pub driver: Signer<'info>,
}

pub fn handler(ctx: Context<AcceptOrder>) -> Result<()> {
    let order = &mut ctx.accounts.order;

    // Order must be fully funded before a driver can accept
    require!(
        order.status == OrderStatus::Funded,
        ForkitError::InvalidOrderStatus
    );

    order.driver = ctx.accounts.driver.key();
    order.status = OrderStatus::Preparing;

    emit!(OrderAccepted {
        order_id: order.order_id,
        driver: order.driver,
    });

    Ok(())
}
