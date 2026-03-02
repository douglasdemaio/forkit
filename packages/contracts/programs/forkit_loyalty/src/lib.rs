use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("ForkLoy111111111111111111111111111111111");

/// ForkIt Loyalty Program — Phase 2 stub
/// Will manage the $FORK token mint, earning, and redemption.
#[program]
pub mod forkit_loyalty {
    use super::*;

    pub fn earn_points(ctx: Context<EarnPoints>, amount: u64) -> Result<()> {
        instructions::earn_points::handler(ctx, amount)
    }

    pub fn redeem_points(ctx: Context<RedeemPoints>, amount: u64) -> Result<()> {
        instructions::redeem_points::handler(ctx, amount)
    }
}
