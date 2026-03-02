use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct UpdateLoyaltyPoints<'info> {
    #[account(mut)]
    pub profile: Account<'info, Profile>,
    /// In production, this should be constrained to the escrow program via CPI
    pub authority: Signer<'info>,
}

pub fn handler(ctx: Context<UpdateLoyaltyPoints>, points_delta: i64) -> Result<()> {
    let profile = &mut ctx.accounts.profile;

    if points_delta >= 0 {
        profile.loyalty_points = profile.loyalty_points.saturating_add(points_delta as u64);
    } else {
        profile.loyalty_points = profile.loyalty_points.saturating_sub(points_delta.unsigned_abs());
    }

    Ok(())
}
