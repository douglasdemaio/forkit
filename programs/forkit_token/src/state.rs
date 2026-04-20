use anchor_lang::prelude::*;
use crate::constants::ORACLE_TWAP_SAMPLES;

// ── TokenConfig — global program configuration ───────────────────────────────

#[account]
pub struct TokenConfig {
    pub mint: Pubkey,                  // FORKIT mint address (PDA)
    pub owner: Pubkey,                 // Can adjust mint rate (7-day timelock)
    pub mint_rate: u64,                // FORKIT units per 1 USDC unit of fee (default 100)
    pub pending_mint_rate: u64,        // Proposed new rate (0 = no pending change)
    pub rate_change_proposed_at: i64,  // Timestamp of proposal (0 = none)
    pub total_supply: u64,             // Running total of all FORKIT minted
    pub global_obligation_count: u64,  // Unexecuted obligations since last batch
    pub last_batch_at: i64,            // Timestamp of last batch execution
    pub locked: bool,                  // Reentrancy guard
    pub bump: u8,
    pub mint_bump: u8,
}

impl TokenConfig {
    pub const SPACE: usize = 8 + 32 + 32 + 8 + 8 + 8 + 8 + 8 + 8 + 1 + 1 + 1 + 16; // +padding
}

// ── ReserveConfig — basket allocations and multisig state ────────────────────

#[account]
pub struct ReserveConfig {
    /// Current target allocation basis points (must sum to 10_000)
    pub usdc_bps: u16,
    pub sol_bps: u16,
    pub btc_bps: u16,
    /// Pending allocations after a successful governance vote (effective Jan 1)
    pub pending_usdc_bps: u16,
    pub pending_sol_bps: u16,
    pub pending_btc_bps: u16,
    pub pending_effective_at: i64,     // Jan 1 following successful vote (0 = none)
    /// 3-of-5 multisig for reserve withdrawals
    pub multisig_signers: [Pubkey; 5],
    pub withdrawal_nonce: u64,         // Incremented on each new withdrawal request
    pub locked: bool,
    pub bump: u8,
}

impl ReserveConfig {
    pub const SPACE: usize = 8 + 2 + 2 + 2 + 2 + 2 + 2 + 8 + (32 * 5) + 8 + 1 + 1 + 16;
}

// ── MintObligation — per-user accumulator for pending FORKIT mints ───────────

#[account]
pub struct MintObligation {
    pub wallet: Pubkey,
    pub pending_forkit: u64,   // FORKIT tokens (6 decimals) to be minted on next batch
    pub bump: u8,
}

impl MintObligation {
    pub const SPACE: usize = 8 + 32 + 8 + 1 + 8;
}

// ── OraclePrice — TWAP via rolling 8-sample hourly average ───────────────────
//
// Each call to `update_oracle_price` adds a new sample (rate-limited to once
// per hour). The TWAP returned by `twap_price()` is the mean of all non-zero
// entries, providing an 8-hour time-weighted average that resists single-block
// price manipulation.

#[account]
pub struct OraclePrice {
    pub symbol: [u8; 8],                       // e.g. b"SOL/USD\0"
    pub price_samples: [u64; ORACLE_TWAP_SAMPLES], // micro-USD per whole token
    pub sample_idx: u8,                        // write position in circular buffer
    pub last_updated: i64,                     // unix timestamp of latest sample
    pub oracle_updater: Pubkey,                // authorized update authority
    pub bump: u8,
}

impl OraclePrice {
    pub const SPACE: usize = 8 + 8 + (8 * ORACLE_TWAP_SAMPLES) + 1 + 8 + 32 + 1 + 16;

    /// Time-weighted average price in micro-USD (6 decimal places).
    pub fn twap_price(&self) -> u64 {
        let valid: Vec<u64> = self.price_samples.iter().copied().filter(|&p| p > 0).collect();
        if valid.is_empty() {
            return 0;
        }
        valid.iter().sum::<u64>() / valid.len() as u64
    }

    /// Add a new price sample and advance the circular buffer.
    pub fn push_sample(&mut self, price: u64, timestamp: i64) {
        let idx = self.sample_idx as usize % ORACLE_TWAP_SAMPLES;
        self.price_samples[idx] = price;
        self.sample_idx = self.sample_idx.wrapping_add(1);
        self.last_updated = timestamp;
    }
}

// ── GovernanceState — annual reserve-rebalancing vote ────────────────────────

#[account]
pub struct GovernanceState {
    pub year: u32,
    /// Proposed new allocations (submitted when voting opens)
    pub proposal_usdc_bps: u16,
    pub proposal_sol_bps: u16,
    pub proposal_btc_bps: u16,
    /// Accumulated vote weights
    pub total_approve_weight: u64,
    pub total_reject_weight: u64,
    /// Total FORKIT supply at the Oct-31 snapshot (used for quorum check)
    pub total_supply_at_snapshot: u64,
    pub vote_finalized: bool,
    pub quorum_met: bool,
    pub passed: bool,
    pub bump: u8,
}

impl GovernanceState {
    pub const SPACE: usize = 8 + 4 + 2 + 2 + 2 + 8 + 8 + 8 + 1 + 1 + 1 + 1 + 8;
}

// ── VoteRecord — per-voter per-year record ────────────────────────────────────

#[account]
pub struct VoteRecord {
    pub voter: Pubkey,
    pub year: u32,
    pub weight: u64,   // FORKIT balance at vote time
    pub approve: bool,
    pub voted: bool,
    pub bump: u8,
}

impl VoteRecord {
    pub const SPACE: usize = 8 + 32 + 4 + 8 + 1 + 1 + 1 + 8;
}

// ── WithdrawalRequest — 3-of-5 multisig for reserve withdrawals ───────────────

#[account]
pub struct WithdrawalRequest {
    pub nonce: u64,
    pub recipient: Pubkey,
    pub amount: u64,
    pub token_mint: Pubkey,
    /// Bitmask: bit i = signer i (from ReserveConfig.multisig_signers) has approved
    pub approval_bitmask: u8,
    pub executed: bool,
    pub created_at: i64,
    pub bump: u8,
}

impl WithdrawalRequest {
    pub const SPACE: usize = 8 + 8 + 32 + 8 + 32 + 1 + 1 + 8 + 1 + 8;

    pub fn approvals_count(&self) -> u8 {
        self.approval_bitmask.count_ones() as u8
    }
}

// ── Events ────────────────────────────────────────────────────────────────────

#[event]
pub struct ForkitTokenInitialized {
    pub mint: Pubkey,
    pub owner: Pubkey,
}

#[event]
pub struct FeesCollected {
    pub customer: Pubkey,
    pub payment_amount: u64,
    pub platform_fee: u64,
    pub restaurant_fee: u64,
    pub reserve_fee: u64,
    pub customer_fee_usdc: u64,       // USDC routed to reserve as backing
    pub forkit_obligation: u64,        // FORKIT tokens queued for customer
}

#[event]
pub struct TokensMinted {
    pub customer: Pubkey,
    pub amount_minted: u64,
    pub new_total_supply: u64,
    pub obligation_count_at_trigger: u64,
}

#[event]
pub struct OraclePriceUpdated {
    pub symbol: [u8; 8],
    pub price_micro_usd: u64,
    pub twap_micro_usd: u64,
    pub timestamp: i64,
}

#[event]
pub struct ReserveUpdated {
    pub usdc_balance: u64,
    pub sol_balance: u64,
    pub btc_balance: u64,
    pub total_value_micro_usd: u64,
    pub target_usdc_bps: u16,
    pub target_sol_bps: u16,
    pub target_btc_bps: u16,
}

#[event]
pub struct VotingOpened {
    pub year: u32,
    pub proposal_usdc_bps: u16,
    pub proposal_sol_bps: u16,
    pub proposal_btc_bps: u16,
    pub total_supply_snapshot: u64,
}

#[event]
pub struct VoteCast {
    pub year: u32,
    pub voter: Pubkey,
    pub weight: u64,
    pub approve: bool,
}

#[event]
pub struct GovernanceVoteResult {
    pub year: u32,
    pub final_usdc_bps: u16,
    pub final_sol_bps: u16,
    pub final_btc_bps: u16,
    pub total_votes_cast: u64,       // weighted sum of all votes
    pub participation_bps: u64,      // (votes / supply) * 10_000
    pub quorum_met: bool,
    pub passed: bool,
}

#[event]
pub struct MintRateChangeProposed {
    pub proposed_rate: u64,
    pub effective_at: i64,
}

#[event]
pub struct MintRateChanged {
    pub old_rate: u64,
    pub new_rate: u64,
}

#[event]
pub struct WithdrawalRequested {
    pub nonce: u64,
    pub recipient: Pubkey,
    pub amount: u64,
    pub token_mint: Pubkey,
}

#[event]
pub struct WithdrawalApproved {
    pub nonce: u64,
    pub signer: Pubkey,
    pub approvals_count: u8,
}

#[event]
pub struct WithdrawalExecuted {
    pub nonce: u64,
    pub recipient: Pubkey,
    pub amount: u64,
    pub token_mint: Pubkey,
}
