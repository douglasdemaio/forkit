use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::ForkitError;
use forkit_registry::state::{Profile, Role};

/// Minimum trust score (out of 10 000) a driver must hold to accept an order.
/// 1 000 = 10.00 — a low floor that still blocks brand-new unrated accounts.
pub const MIN_DRIVER_TRUST_SCORE: u32 = 1_000;

#[derive(Accounts)]
pub struct AcceptOrder<'info> {
    #[account(
        mut,
        seeds = [Order::SEED, &order.order_id.to_le_bytes()],
        bump = order.bump,
    )]
    pub order: Account<'info, Order>,

    /// Driver's registry profile — must exist, be Role::Driver, active, and
    /// hold at least MIN_DRIVER_TRUST_SCORE. PDA seeds verified so the account
    /// cannot be spoofed with an arbitrary Profile belonging to another wallet.
    #[account(
        seeds = [
            Profile::SEED,
            driver.key().as_ref(),
            &[Role::Driver as u8],
        ],
        bump = driver_profile.bump,
        constraint = driver_profile.role == Role::Driver
            @ ForkitError::DriverNotRegistered,
        constraint = driver_profile.is_active
            @ ForkitError::DriverNotActive,
        constraint = driver_profile.trust_score >= MIN_DRIVER_TRUST_SCORE
            @ ForkitError::InsufficientTrustScore,
        seeds::program = forkit_registry::ID,
    )]
    pub driver_profile: Account<'info, Profile>,

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
