use anchor_lang::prelude::*;
use crate::state::{ProtocolConfig, SurgeConfig, SurgeUpdated, MAX_SURGE_MULTIPLIER_BPS};
use crate::errors::ForkitError;

#[derive(Accounts)]
pub struct SetSurgePricing<'info> {
    #[account(
        init_if_needed,
        payer = admin,
        space = SurgeConfig::SPACE,
        seeds = [SurgeConfig::SEED],
        bump,
    )]
    pub surge_config: Account<'info, SurgeConfig>,

    #[account(
        seeds = [ProtocolConfig::SEED],
        bump = protocol_config.bump,
        has_one = admin @ ForkitError::Unauthorized,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,

    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Set or update the AI surge-pricing multiplier.
///
/// Called by the ForkIt backend AI pricing engine whenever demand forecasts
/// indicate a supply/demand imbalance (e.g., lunch rush, bad weather, high-demand zone).
///
/// - `multiplier_bps`: extra basis points added to base delivery fee (max 30 000 = 3×).
/// - `active`: toggle surge on/off without resetting the multiplier value.
///
/// Emits `SurgeUpdated` for real-time frontend price-warning display.
pub fn handler(
    ctx: Context<SetSurgePricing>,
    multiplier_bps: u16,
    active: bool,
) -> Result<()> {
    require!(
        multiplier_bps <= MAX_SURGE_MULTIPLIER_BPS,
        ForkitError::SurgeMultiplierTooHigh
    );

    let clock = Clock::get()?;
    let config = &mut ctx.accounts.surge_config;

    config.admin = ctx.accounts.admin.key();
    config.multiplier_bps = multiplier_bps;
    config.active = active;
    config.updated_at = clock.unix_timestamp;
    config.bump = ctx.bumps.surge_config;

    emit!(SurgeUpdated {
        multiplier_bps,
        active,
        updated_at: clock.unix_timestamp,
    });

    Ok(())
}
