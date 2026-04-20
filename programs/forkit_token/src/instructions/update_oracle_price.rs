use anchor_lang::prelude::*;
use crate::state::*;
use crate::constants::*;
use crate::errors::TokenError;

/// Register or update an on-chain TWAP oracle price for a reserve asset.
///
/// The oracle_updater (set at init) pushes hourly spot prices. The program
/// maintains an 8-sample circular buffer; `twap_price()` returns the mean,
/// giving an 8-hour time-weighted average resistant to single-block manipulation.
///
/// Rate-limited to once per ORACLE_UPDATE_INTERVAL (1 hour) per oracle symbol.
///
/// Accepted symbols: "SOL/USD\0", "BTC/USD\0" (8-byte, null-padded ASCII).
#[derive(Accounts)]
#[instruction(symbol: [u8; 8])]
pub struct UpdateOraclePrice<'info> {
    #[account(
        init_if_needed,
        payer = oracle_updater,
        space = OraclePrice::SPACE,
        seeds = [ORACLE_SEED, &symbol],
        bump,
    )]
    pub oracle: Account<'info, OraclePrice>,

    #[account(
        mut,
        constraint = (oracle.oracle_updater == oracle_updater.key()
            || oracle.oracle_updater == Pubkey::default())
            @ TokenError::Unauthorized,
    )]
    pub oracle_updater: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<UpdateOraclePrice>,
    symbol: [u8; 8],
    price_micro_usd: u64,
) -> Result<()> {
    let clock = Clock::get()?;
    let oracle = &mut ctx.accounts.oracle;

    // ── Rate limiting ─────────────────────────────────────────────────────────
    require!(price_micro_usd > 0, TokenError::InvalidOraclePrice);
    if oracle.last_updated > 0 {
        require!(
            clock.unix_timestamp - oracle.last_updated >= ORACLE_UPDATE_INTERVAL,
            TokenError::OracleUpdateTooFrequent
        );
    }

    // ── Initialise fields on first update ────────────────────────────────────
    if oracle.oracle_updater == Pubkey::default() {
        oracle.symbol = symbol;
        oracle.oracle_updater = ctx.accounts.oracle_updater.key();
        oracle.bump = ctx.bumps.oracle;
    }

    oracle.push_sample(price_micro_usd, clock.unix_timestamp);

    let twap = oracle.twap_price();
    emit!(OraclePriceUpdated {
        symbol,
        price_micro_usd,
        twap_micro_usd: twap,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}
