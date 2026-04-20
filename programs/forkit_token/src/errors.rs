use anchor_lang::prelude::*;

#[error_code]
pub enum TokenError {
    #[msg("Fee splits do not sum to exactly 0.02% of the payment amount")]
    FeeSplitMismatch,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Unauthorized: caller is not the program owner")]
    Unauthorized,
    #[msg("Reentrancy guard triggered")]
    Reentrancy,
    #[msg("Unsupported payment mint")]
    UnsupportedMint,
    #[msg("Invalid platform wallet")]
    InvalidPlatformWallet,
    #[msg("Mint batch threshold not yet reached (neither tx count nor time elapsed)")]
    BatchNotReady,
    #[msg("No pending FORKIT obligations for this wallet")]
    NoPendingObligation,
    #[msg("Oracle update too frequent — must wait at least 1 hour between updates")]
    OracleUpdateTooFrequent,
    #[msg("Oracle price is stale — price must be updated within the last 2 hours")]
    StalePriceOracle,
    #[msg("Oracle price is zero or negative")]
    InvalidOraclePrice,
    #[msg("Oracle symbol not found")]
    UnknownOracleSymbol,
    #[msg("Reserve allocation percentages must sum to exactly 10 000 basis points")]
    AllocationSumMismatch,
    #[msg("Voting window is not open for this year (Oct 31 – Nov 7 UTC)")]
    VotingWindowClosed,
    #[msg("Governance vote for this year has already been finalized")]
    VoteAlreadyFinalized,
    #[msg("This wallet has already voted for this year")]
    AlreadyVoted,
    #[msg("Voter has no FORKIT balance to cast a vote")]
    ZeroVotingWeight,
    #[msg("Voting has not yet closed (must be after Nov 7 23:59 UTC)")]
    VotingStillOpen,
    #[msg("A single-asset allocation exceeding 60% requires a 60% supermajority")]
    SupermajorityRequired,
    #[msg("Rate change timelock has not expired yet (7 days required)")]
    TimelockNotExpired,
    #[msg("No pending rate change to execute")]
    NoPendingRateChange,
    #[msg("New mint rate must be greater than zero")]
    InvalidMintRate,
    #[msg("Caller is not a registered multisig signer")]
    NotMultisigSigner,
    #[msg("Withdrawal has already been executed")]
    WithdrawalAlreadyExecuted,
    #[msg("Insufficient approvals — 3-of-5 required for reserve withdrawals")]
    InsufficientApprovals,
    #[msg("This signer has already approved this withdrawal")]
    AlreadyApproved,
    #[msg("Governance state for this year already exists")]
    GovernanceAlreadyOpen,
    #[msg("Voting has not opened for this year yet")]
    VotingNotYetOpen,
    #[msg("New allocation would give a single asset >60% without required supermajority")]
    InvalidAllocation,
}
