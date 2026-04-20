use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::*;
use crate::constants::*;
use crate::errors::TokenError;

// Oracle symbols
const SOL_USD_SYMBOL: [u8; 8] = *b"SOL/USD\0";
const BTC_USD_SYMBOL: [u8; 8] = *b"BTC/USD\0";

// Price staleness limit: oracle must have been updated within 2 hours
const MAX_ORACLE_AGE_SECS: i64 = 7_200;

/// Read reserve vault balances, compute current allocation via TWAP oracle
/// prices, compare against the target basket, and calculate the rebalancing
/// amounts needed.
///
/// The actual DEX swap is intentionally left as a TODO — in production this
/// would issue a Jupiter Aggregator CPI (via remaining_accounts) using the
/// TWAP price to set a minimum-out floor, preventing slippage attacks.
///
/// Anyone can call this instruction; it is permissionless and idempotent.
/// Emits a ReserveUpdated event that off-chain keepers can use to trigger swaps.
#[derive(Accounts)]
pub struct ConvertReserve<'info> {
    #[account(
        mut,
        seeds = [RESERVE_CONFIG_SEED],
        bump = reserve_config.bump,
        constraint = !reserve_config.locked @ TokenError::Reentrancy,
    )]
    pub reserve_config: Account<'info, ReserveConfig>,

    #[account(
        seeds = [TOKEN_CONFIG_SEED],
        bump = token_config.bump,
    )]
    pub token_config: Account<'info, TokenConfig>,

    /// USDC reserve vault
    #[account(
        seeds = [RESERVE_VAULT_SEED, usdc_mint.key().as_ref()],
        bump,
    )]
    pub reserve_usdc_vault: Account<'info, TokenAccount>,

    /// wSOL reserve vault (may be empty — balance read as 0 if so)
    /// CHECK: optional vault; balance read directly from lamports/token balance
    #[account(
        seeds = [RESERVE_VAULT_SEED, wsol_mint.key().as_ref()],
        bump,
    )]
    pub reserve_sol_vault: Account<'info, TokenAccount>,

    /// wBTC reserve vault (may be empty)
    #[account(
        seeds = [RESERVE_VAULT_SEED, wbtc_mint.key().as_ref()],
        bump,
    )]
    pub reserve_btc_vault: Account<'info, TokenAccount>,

    pub usdc_mint: Account<'info, Mint>,
    pub wsol_mint: Account<'info, Mint>,
    pub wbtc_mint: Account<'info, Mint>,

    /// SOL/USD TWAP oracle account
    #[account(
        seeds = [ORACLE_SEED, &SOL_USD_SYMBOL],
        bump = sol_oracle.bump,
    )]
    pub sol_oracle: Account<'info, OraclePrice>,

    /// BTC/USD TWAP oracle account
    #[account(
        seeds = [ORACLE_SEED, &BTC_USD_SYMBOL],
        bump = btc_oracle.bump,
    )]
    pub btc_oracle: Account<'info, OraclePrice>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<ConvertReserve>) -> Result<()> {
    let clock = Clock::get()?;
    ctx.accounts.reserve_config.locked = true;

    // ── Activate pending governance allocations if effective date passed ───────
    let reserve = &mut ctx.accounts.reserve_config;
    if reserve.pending_effective_at > 0 && clock.unix_timestamp >= reserve.pending_effective_at {
        reserve.usdc_bps = reserve.pending_usdc_bps;
        reserve.sol_bps = reserve.pending_sol_bps;
        reserve.btc_bps = reserve.pending_btc_bps;
        reserve.pending_effective_at = 0;
    }

    // ── Validate oracle freshness ─────────────────────────────────────────────
    require!(
        clock.unix_timestamp - ctx.accounts.sol_oracle.last_updated <= MAX_ORACLE_AGE_SECS,
        TokenError::StalePriceOracle
    );
    require!(
        clock.unix_timestamp - ctx.accounts.btc_oracle.last_updated <= MAX_ORACLE_AGE_SECS,
        TokenError::StalePriceOracle
    );

    let sol_price = ctx.accounts.sol_oracle.twap_price();
    let btc_price = ctx.accounts.btc_oracle.twap_price();
    require!(sol_price > 0, TokenError::InvalidOraclePrice);
    require!(btc_price > 0, TokenError::InvalidOraclePrice);

    // ── Current vault balances ────────────────────────────────────────────────
    let usdc_balance = ctx.accounts.reserve_usdc_vault.amount;
    let sol_balance = ctx.accounts.reserve_sol_vault.amount;
    let btc_balance = ctx.accounts.reserve_btc_vault.amount;

    // ── Compute total USD value via TWAP (all in micro-USD units) ─────────────
    // usdc: 6 decimals → value = balance (already micro-USD)
    // sol: 9 decimals → value = balance / 1_000 * sol_price_per_unit
    // btc: 8 decimals → value = balance / 100 * btc_price_per_unit
    let usdc_value = usdc_balance as u128;

    let sol_value = (sol_balance as u128)
        .checked_mul(sol_price as u128)
        .unwrap_or(0)
        .checked_div(1_000_000_000) // wSOL has 9 decimals
        .unwrap_or(0);

    let btc_value = (btc_balance as u128)
        .checked_mul(btc_price as u128)
        .unwrap_or(0)
        .checked_div(100_000_000) // wBTC has 8 decimals
        .unwrap_or(0);

    let total_value = usdc_value
        .checked_add(sol_value).unwrap_or(u128::MAX)
        .checked_add(btc_value).unwrap_or(u128::MAX);

    // ── Compute actual allocation bps for each asset ──────────────────────────
    let (usdc_bps_actual, sol_bps_actual, btc_bps_actual) = if total_value == 0 {
        (10_000u64, 0u64, 0u64)
    } else {
        (
            usdc_value.checked_mul(10_000).unwrap_or(0).checked_div(total_value).unwrap_or(0),
            sol_value.checked_mul(10_000).unwrap_or(0).checked_div(total_value).unwrap_or(0),
            btc_value.checked_mul(10_000).unwrap_or(0).checked_div(total_value).unwrap_or(0),
        )
    };

    // ── Check if rebalancing is needed (any asset drifts > REBALANCE_DEVIATION_BPS) ──
    let reserve = &ctx.accounts.reserve_config;
    let _needs_rebalance = deviation(usdc_bps_actual, reserve.usdc_bps as u64) > REBALANCE_DEVIATION_BPS
        || deviation(sol_bps_actual, reserve.sol_bps as u64) > REBALANCE_DEVIATION_BPS
        || deviation(btc_bps_actual, reserve.btc_bps as u64) > REBALANCE_DEVIATION_BPS;

    // TODO: Execute DEX swap via Jupiter Aggregator CPI using remaining_accounts.
    //
    // When rebalancing is needed:
    //   1. Compute USDC_to_sell = total_value * target_sol_bps / 10_000 - sol_value  (in USD)
    //      Enforce minimum-out using TWAP price with MAX_SLIPPAGE_BPS tolerance.
    //   2. CPI into Jupiter:
    //      anchor_lang::solana_program::program::invoke_signed(
    //          &jupiter_ix,
    //          &remaining_accounts,
    //          &[token_config_seeds],
    //      )?;
    //   3. Re-read vault balances post-swap and verify they moved in the right direction.
    //
    // NEVER use spot price for the minimum-out — always use the TWAP stored here.

    emit!(ReserveUpdated {
        usdc_balance,
        sol_balance,
        btc_balance,
        total_value_micro_usd: total_value.min(u64::MAX as u128) as u64,
        target_usdc_bps: reserve.usdc_bps,
        target_sol_bps: reserve.sol_bps,
        target_btc_bps: reserve.btc_bps,
    });

    ctx.accounts.reserve_config.locked = false;
    Ok(())
}

fn deviation(actual: u64, target: u64) -> u64 {
    if actual > target { actual - target } else { target - actual }
}
