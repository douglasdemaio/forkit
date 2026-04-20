use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::RegistryError;

#[derive(Accounts)]
pub struct RateCounterparty<'info> {
    /// PDA seeds verified so an attacker cannot supply an arbitrary Profile account.
    /// Self-rating blocked via inline constraint.
    #[account(
        mut,
        seeds = [Profile::SEED, target_profile.wallet.as_ref(), &[target_profile.role as u8]],
        bump = target_profile.bump,
        constraint = target_profile.wallet != rater.key() @ RegistryError::SelfRatingNotAllowed,
    )]
    pub target_profile: Account<'info, Profile>,
    pub rater: Signer<'info>,
}

pub fn handler(ctx: Context<RateCounterparty>, rating: u8) -> Result<()> {
    require!(rating >= 1 && rating <= 5, RegistryError::InvalidRating);

    let clock = Clock::get()?;
    let profile = &mut ctx.accounts.target_profile;

    profile.total_ratings = profile.total_ratings
        .checked_add(1)
        .ok_or(RegistryError::ArithmeticOverflow)?;
    profile.sum_ratings = profile.sum_ratings
        .checked_add(rating as u64)
        .ok_or(RegistryError::ArithmeticOverflow)?;
    profile.last_active_at = clock.unix_timestamp;
    profile.recalculate_trust(clock.unix_timestamp);

    emit!(ProfileRated {
        target: profile.wallet,
        rating,
        new_trust_score: profile.trust_score,
    });

    Ok(())
}
