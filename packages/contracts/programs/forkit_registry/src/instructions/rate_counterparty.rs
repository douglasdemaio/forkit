use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct RateCounterparty<'info> {
    #[account(mut)]
    pub target_profile: Account<'info, Profile>,
    pub rater: Signer<'info>,
}

pub fn handler(ctx: Context<RateCounterparty>, rating: u8) -> Result<()> {
    require!(rating >= 1 && rating <= 5, ErrorCode::ConstraintRaw);

    let clock = Clock::get()?;
    let profile = &mut ctx.accounts.target_profile;

    profile.total_ratings += 1;
    profile.sum_ratings += rating as u64;
    profile.last_active_at = clock.unix_timestamp;
    profile.recalculate_trust(clock.unix_timestamp);

    emit!(ProfileRated {
        target: profile.wallet,
        rating,
        new_trust_score: profile.trust_score,
    });

    Ok(())
}
