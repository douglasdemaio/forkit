use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;
use crate::errors::TokenError;

// ── propose_rate_change ───────────────────────────────────────────────────────

/// Propose a new FORKIT mint rate. The change will not take effect until
/// RATE_CHANGE_TIMELOCK (7 days) has elapsed, giving the community time to
/// observe and react. Only the program owner may propose rate changes.
#[derive(Accounts)]
pub struct ProposeRateChange<'info> {
    #[account(
        mut,
        seeds = [TOKEN_CONFIG_SEED],
        bump = token_config.bump,
        has_one = owner @ TokenError::Unauthorized,
    )]
    pub token_config: Account<'info, TokenConfig>,

    pub owner: Signer<'info>,
}

pub fn propose_rate_change_handler(
    ctx: Context<ProposeRateChange>,
    new_rate: u64,
) -> Result<()> {
    require!(new_rate > 0, TokenError::InvalidMintRate);

    let clock = Clock::get()?;
    let config = &mut ctx.accounts.token_config;
    config.pending_mint_rate = new_rate;
    config.rate_change_proposed_at = clock.unix_timestamp;

    let effective_at = clock.unix_timestamp + RATE_CHANGE_TIMELOCK;
    emit!(MintRateChangeProposed {
        proposed_rate: new_rate,
        effective_at,
    });

    Ok(())
}

// ── execute_rate_change ───────────────────────────────────────────────────────

/// Apply a previously proposed mint rate change after the 7-day timelock.
/// Permissionless — anyone can call once the timelock expires.
#[derive(Accounts)]
pub struct ExecuteRateChange<'info> {
    #[account(
        mut,
        seeds = [TOKEN_CONFIG_SEED],
        bump = token_config.bump,
    )]
    pub token_config: Account<'info, TokenConfig>,

    pub caller: Signer<'info>,
}

pub fn execute_rate_change_handler(ctx: Context<ExecuteRateChange>) -> Result<()> {
    let clock = Clock::get()?;
    let config = &mut ctx.accounts.token_config;

    require!(config.pending_mint_rate > 0, TokenError::NoPendingRateChange);
    require!(
        clock.unix_timestamp >= config.rate_change_proposed_at + RATE_CHANGE_TIMELOCK,
        TokenError::TimelockNotExpired
    );

    let old_rate = config.mint_rate;
    config.mint_rate = config.pending_mint_rate;
    config.pending_mint_rate = 0;
    config.rate_change_proposed_at = 0;

    emit!(MintRateChanged {
        old_rate,
        new_rate: config.mint_rate,
    });

    Ok(())
}
