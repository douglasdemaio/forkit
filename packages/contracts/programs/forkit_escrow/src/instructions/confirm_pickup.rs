use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::hash;
use crate::state::*;
use crate::errors::ForkitError;

#[derive(Accounts)]
pub struct ConfirmPickup<'info> {
    #[account(
        mut,
        seeds = [Order::SEED, &order.order_id.to_le_bytes()],
        bump = order.bump,
        has_one = driver @ ForkitError::Unauthorized,
    )]
    pub order: Account<'info, Order>,
    pub driver: Signer<'info>,
}

pub fn handler(ctx: Context<ConfirmPickup>, code_a: String) -> Result<()> {
    let order = &mut ctx.accounts.order;

    require!(
        order.status == OrderStatus::ReadyForPickup,
        ForkitError::InvalidOrderStatus
    );

    // Verify CODE_A by hashing and comparing
    let code_hash = hash(code_a.as_bytes());
    require!(
        code_hash.to_bytes() == order.code_a_hash,
        ForkitError::InvalidCode
    );

    let clock = Clock::get()?;
    order.status = OrderStatus::PickedUp;
    order.pickup_confirmed_at = clock.unix_timestamp;

    emit!(PickupConfirmed {
        order_id: order.order_id,
        driver: order.driver,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
