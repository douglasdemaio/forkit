use anchor_lang::prelude::Pubkey;

// ── Protocol wallet (immutable) ─────────────────────────────────────────────
// Full macro path required — `use ...::pubkey` imports the module, not the macro.
pub const PLATFORM_WALLET: Pubkey =
    anchor_lang::solana_program::pubkey!("9iBQEn9yMbKVhJKEpMpPByS6pjydPmQDGaznMaCvGkzD");

// ── Fee distribution (parts per million, must sum to TOTAL_FEE_PPM) ─────────
// 0.02% total = 200 ppm; split equally four ways = 50 ppm each
pub const TOTAL_FEE_PPM: u64 = 200; // 0.02%
pub const PLATFORM_FEE_PPM: u64 = 50; // 0.005%
pub const CUSTOMER_FEE_PPM: u64 = 50; // 0.005% → triggers FORKIT mint
pub const RESTAURANT_FEE_PPM: u64 = 50; // 0.005%
pub const RESERVE_FEE_PPM: u64 = 50; // 0.005% → reserve basket

// ── Mint rate: FORKIT tokens per unit of customer fee (6-decimal units) ──────
// 1 FORKIT per $0.01 of fee ≡ 100 FORKIT per $1 ≡ 100 units per USDC unit
pub const DEFAULT_MINT_RATE: u64 = 100;

// ── Batched minting thresholds ────────────────────────────────────────────────
pub const MINT_BATCH_TX_THRESHOLD: u64 = 100; // execute batch after N obligations
pub const MINT_BATCH_TIME_THRESHOLD: i64 = 3_600; // …or 1 hour, whichever first

// ── Governance ────────────────────────────────────────────────────────────────
pub const GOVERNANCE_QUORUM_BPS: u64 = 1_000; // 10% of supply must participate
pub const GOVERNANCE_MAJORITY_BPS: u64 = 5_000; // >50% of votes must approve
pub const SUPERMAJORITY_BPS: u64 = 6_000; // 60% required for >60% single-asset
pub const MAX_SINGLE_ASSET_BPS: u16 = 6_000; // single asset allocation ceiling

// ── Timelock ──────────────────────────────────────────────────────────────────
pub const RATE_CHANGE_TIMELOCK: i64 = 7 * 24 * 3_600; // 7 days

// ── Multisig ──────────────────────────────────────────────────────────────────
pub const MULTISIG_THRESHOLD: u8 = 3; // 3-of-5 required for reserve withdrawals
pub const MULTISIG_SIGNERS: usize = 5;

// ── Token parameters ─────────────────────────────────────────────────────────
pub const FORKIT_DECIMALS: u8 = 6;
pub const TOTAL_BPS: u16 = 10_000;

// ── PDA seeds ────────────────────────────────────────────────────────────────
pub const TOKEN_CONFIG_SEED: &[u8] = b"token_config";
pub const FORKIT_MINT_SEED: &[u8] = b"forkit_mint";
pub const RESERVE_CONFIG_SEED: &[u8] = b"reserve_config";
pub const RESERVE_VAULT_SEED: &[u8] = b"reserve_vault";
pub const MINT_OBLIGATION_SEED: &[u8] = b"mint_obligation";
pub const ORACLE_SEED: &[u8] = b"oracle";
pub const GOVERNANCE_SEED: &[u8] = b"governance";
pub const VOTE_RECORD_SEED: &[u8] = b"vote_record";
pub const WITHDRAWAL_SEED: &[u8] = b"withdrawal";

// ── Default reserve target allocation (basis points out of 10 000) ────────────
pub const DEFAULT_RESERVE_USDC_BPS: u16 = 5_000; // 50%
pub const DEFAULT_RESERVE_SOL_BPS: u16 = 3_000; // 30%
pub const DEFAULT_RESERVE_BTC_BPS: u16 = 2_000; // 20%

// ── Oracle rate-limit ─────────────────────────────────────────────────────────
pub const ORACLE_UPDATE_INTERVAL: i64 = 3_600; // max one price update per hour
pub const ORACLE_TWAP_SAMPLES: usize = 8; // 8-hour rolling average = TWAP window

// ── Reserve rebalance deviation threshold ─────────────────────────────────────
pub const REBALANCE_DEVIATION_BPS: u64 = 500; // rebalance if any asset drifts >5%
