use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount};
use crate::state::*;
use crate::constants::*;
use crate::errors::TokenError;

// ── Date helpers ──────────────────────────────────────────────────────────────

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: u32, month: u8) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => if is_leap_year(year) { 29 } else { 28 },
        _ => 0,
    }
}

/// Unix timestamp for 00:00:00 UTC on (year, month, day).
fn timestamp_for_date(year: u32, month: u8, day: u8) -> i64 {
    let y = year as i64;
    let prev_y = y - 1;
    let leap_before = prev_y / 4 - prev_y / 100 + prev_y / 400;
    let base_leap: i64 = 1969 / 4 - 1969 / 100 + 1969 / 400;
    let days_to_year = (y - 1970) * 365 + (leap_before - base_leap);

    let mut days_in_year: i64 = 0;
    for m in 1..month {
        days_in_year += days_in_month(year, m) as i64;
    }

    (days_to_year + days_in_year + (day as i64) - 1) * 86400
}

fn voting_open_ts(year: u32) -> i64 {
    timestamp_for_date(year, 10, 31) // Oct 31 00:00 UTC
}

fn voting_close_ts(year: u32) -> i64 {
    timestamp_for_date(year, 11, 7) + 86_399 // Nov 7 23:59:59 UTC
}

fn allocations_effective_ts(year: u32) -> i64 {
    timestamp_for_date(year + 1, 1, 1) // Jan 1 of following year
}

// ── open_voting ───────────────────────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(year: u32)]
pub struct OpenVoting<'info> {
    #[account(
        init,
        payer = proposer,
        space = GovernanceState::SPACE,
        seeds = [GOVERNANCE_SEED, &year.to_le_bytes()],
        bump,
    )]
    pub governance: Account<'info, GovernanceState>,

    #[account(seeds = [TOKEN_CONFIG_SEED], bump = token_config.bump)]
    pub token_config: Account<'info, TokenConfig>,

    #[account(mut)]
    pub proposer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn open_voting_handler(
    ctx: Context<OpenVoting>,
    year: u32,
    proposal_usdc_bps: u16,
    proposal_sol_bps: u16,
    proposal_btc_bps: u16,
) -> Result<()> {
    let clock = Clock::get()?;

    // ── Validate window ───────────────────────────────────────────────────────
    require!(
        clock.unix_timestamp >= voting_open_ts(year)
            && clock.unix_timestamp <= voting_close_ts(year),
        TokenError::VotingWindowClosed
    );

    // ── Validate allocation sum = 10 000 bps ─────────────────────────────────
    require!(
        (proposal_usdc_bps as u32)
            .checked_add(proposal_sol_bps as u32)
            .and_then(|s| s.checked_add(proposal_btc_bps as u32))
            == Some(TOTAL_BPS as u32),
        TokenError::AllocationSumMismatch
    );

    let governance = &mut ctx.accounts.governance;
    governance.year = year;
    governance.proposal_usdc_bps = proposal_usdc_bps;
    governance.proposal_sol_bps = proposal_sol_bps;
    governance.proposal_btc_bps = proposal_btc_bps;
    governance.total_approve_weight = 0;
    governance.total_reject_weight = 0;
    governance.total_supply_at_snapshot = ctx.accounts.token_config.total_supply;
    governance.vote_finalized = false;
    governance.quorum_met = false;
    governance.passed = false;
    governance.bump = ctx.bumps.governance;

    emit!(VotingOpened {
        year,
        proposal_usdc_bps,
        proposal_sol_bps,
        proposal_btc_bps,
        total_supply_snapshot: ctx.accounts.token_config.total_supply,
    });

    Ok(())
}

// ── cast_vote ─────────────────────────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(year: u32)]
pub struct CastVote<'info> {
    #[account(
        mut,
        seeds = [GOVERNANCE_SEED, &year.to_le_bytes()],
        bump = governance.bump,
        constraint = !governance.vote_finalized @ TokenError::VoteAlreadyFinalized,
    )]
    pub governance: Account<'info, GovernanceState>,

    #[account(
        init,
        payer = voter,
        space = VoteRecord::SPACE,
        seeds = [VOTE_RECORD_SEED, &year.to_le_bytes(), voter.key().as_ref()],
        bump,
    )]
    pub vote_record: Account<'info, VoteRecord>,

    /// Voter's FORKIT ATA — balance is read as voting weight at cast time.
    #[account(
        constraint = voter_forkit_ata.owner == voter.key() @ TokenError::Unauthorized,
    )]
    pub voter_forkit_ata: Account<'info, TokenAccount>,

    #[account(seeds = [FORKIT_MINT_SEED], bump = token_config.mint_bump)]
    pub forkit_mint: Account<'info, Mint>,

    #[account(seeds = [TOKEN_CONFIG_SEED], bump = token_config.bump)]
    pub token_config: Account<'info, TokenConfig>,

    #[account(mut)]
    pub voter: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn cast_vote_handler(
    ctx: Context<CastVote>,
    year: u32,
    approve: bool,
) -> Result<()> {
    let clock = Clock::get()?;

    // ── Validate window is still open ─────────────────────────────────────────
    require!(
        clock.unix_timestamp >= voting_open_ts(year)
            && clock.unix_timestamp <= voting_close_ts(year),
        TokenError::VotingWindowClosed
    );

    let weight = ctx.accounts.voter_forkit_ata.amount;
    require!(weight > 0, TokenError::ZeroVotingWeight);

    // ── Record vote ───────────────────────────────────────────────────────────
    let record = &mut ctx.accounts.vote_record;
    record.voter = ctx.accounts.voter.key();
    record.year = year;
    record.weight = weight;
    record.approve = approve;
    record.voted = true;
    record.bump = ctx.bumps.vote_record;

    let governance = &mut ctx.accounts.governance;
    if approve {
        governance.total_approve_weight = governance.total_approve_weight
            .checked_add(weight)
            .ok_or(TokenError::ArithmeticOverflow)?;
    } else {
        governance.total_reject_weight = governance.total_reject_weight
            .checked_add(weight)
            .ok_or(TokenError::ArithmeticOverflow)?;
    }

    emit!(VoteCast { year, voter: ctx.accounts.voter.key(), weight, approve });

    Ok(())
}

// ── finalize_vote ─────────────────────────────────────────────────────────────

#[derive(Accounts)]
#[instruction(year: u32)]
pub struct FinalizeVote<'info> {
    #[account(
        mut,
        seeds = [GOVERNANCE_SEED, &year.to_le_bytes()],
        bump = governance.bump,
        constraint = !governance.vote_finalized @ TokenError::VoteAlreadyFinalized,
    )]
    pub governance: Account<'info, GovernanceState>,

    #[account(
        mut,
        seeds = [RESERVE_CONFIG_SEED],
        bump = reserve_config.bump,
    )]
    pub reserve_config: Account<'info, ReserveConfig>,

    pub caller: Signer<'info>,
}

pub fn finalize_vote_handler(ctx: Context<FinalizeVote>, year: u32) -> Result<()> {
    let clock = Clock::get()?;

    // ── Voting must have closed ───────────────────────────────────────────────
    require!(
        clock.unix_timestamp > voting_close_ts(year),
        TokenError::VotingStillOpen
    );

    let gov = &mut ctx.accounts.governance;
    gov.vote_finalized = true;

    let total_votes = gov.total_approve_weight
        .checked_add(gov.total_reject_weight)
        .ok_or(TokenError::ArithmeticOverflow)?;

    let supply = gov.total_supply_at_snapshot;

    // ── Quorum check: >= 10% of circulating supply must participate ────────────
    let quorum_met = if supply == 0 {
        false
    } else {
        let participation_bps = (total_votes as u128)
            .checked_mul(10_000)
            .unwrap_or(0)
            .checked_div(supply as u128)
            .unwrap_or(0) as u64;
        participation_bps >= GOVERNANCE_QUORUM_BPS
    };
    gov.quorum_met = quorum_met;

    let participation_bps = if supply == 0 {
        0u64
    } else {
        (total_votes as u128 * 10_000 / supply as u128).min(u64::MAX as u128) as u64
    };

    // ── Majority check: >50% of participating votes must approve ──────────────
    let majority_met = if total_votes == 0 {
        false
    } else {
        let approve_bps = (gov.total_approve_weight as u128)
            .checked_mul(10_000)
            .unwrap_or(0)
            .checked_div(total_votes as u128)
            .unwrap_or(0) as u64;
        approve_bps > GOVERNANCE_MAJORITY_BPS
    };

    // ── Supermajority check: >60% required if any asset > 60% ────────────────
    let needs_supermajority = gov.proposal_usdc_bps > MAX_SINGLE_ASSET_BPS
        || gov.proposal_sol_bps > MAX_SINGLE_ASSET_BPS
        || gov.proposal_btc_bps > MAX_SINGLE_ASSET_BPS;

    let supermajority_met = if needs_supermajority && total_votes > 0 {
        let approve_bps = (gov.total_approve_weight as u128 * 10_000
            / total_votes as u128) as u64;
        approve_bps >= SUPERMAJORITY_BPS
    } else {
        true // supermajority not needed
    };

    let passed = quorum_met && majority_met && supermajority_met;
    gov.passed = passed;

    // ── Apply new allocations (effective Jan 1 of following year) ─────────────
    if passed {
        let reserve = &mut ctx.accounts.reserve_config;
        reserve.pending_usdc_bps = gov.proposal_usdc_bps;
        reserve.pending_sol_bps = gov.proposal_sol_bps;
        reserve.pending_btc_bps = gov.proposal_btc_bps;
        reserve.pending_effective_at = allocations_effective_ts(year);
    }

    let (final_usdc, final_sol, final_btc) = if passed {
        (gov.proposal_usdc_bps, gov.proposal_sol_bps, gov.proposal_btc_bps)
    } else {
        let r = &ctx.accounts.reserve_config;
        (r.usdc_bps, r.sol_bps, r.btc_bps)
    };

    emit!(GovernanceVoteResult {
        year,
        final_usdc_bps: final_usdc,
        final_sol_bps: final_sol,
        final_btc_bps: final_btc,
        total_votes_cast: total_votes,
        participation_bps,
        quorum_met,
        passed,
    });

    Ok(())
}
