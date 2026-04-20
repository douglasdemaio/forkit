use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("FRKTkNq5sGhCVr3QLUwMGaznMaCvGkzNFbPmHd6XbC4q");

#[program]
pub mod forkit_token {
    use super::*;

    /// Deploy the FORKIT SPL token with PDA mint authority, no freeze authority,
    /// and initialise global config + reserve config + USDC reserve vault.
    pub fn initialize(
        ctx: Context<Initialize>,
        multisig_signers: [Pubkey; 5],
    ) -> Result<()> {
        initialize::handler(ctx, multisig_signers)
    }

    /// Collect 0.02% protocol fee from a payment, split it four ways, and
    /// record a FORKIT mint obligation for the customer.
    pub fn collect_fee(ctx: Context<CollectFee>, amount: u64) -> Result<()> {
        collect_fee::handler(ctx, amount)
    }

    /// Mint pending FORKIT tokens for one customer.
    /// Callable by anyone once the batch threshold is met.
    pub fn execute_mint_batch(ctx: Context<ExecuteMintBatch>) -> Result<()> {
        execute_mint_batch::handler(ctx)
    }

    /// Push a new hourly spot price for a reserve asset (rate-limited).
    /// The program stores an 8-sample rolling average as the TWAP.
    pub fn update_oracle_price(
        ctx: Context<UpdateOraclePrice>,
        symbol: [u8; 8],
        price_micro_usd: u64,
    ) -> Result<()> {
        update_oracle_price::handler(ctx, symbol, price_micro_usd)
    }

    /// Read current reserve balances via TWAP oracle prices, calculate
    /// allocation drift, and emit ReserveUpdated for off-chain keepers.
    pub fn convert_reserve(ctx: Context<ConvertReserve>) -> Result<()> {
        convert_reserve::handler(ctx)
    }

    /// Open the annual governance vote (Oct 31 – Nov 7 UTC).
    /// Submits a proposal for next year's reserve basket.
    pub fn open_voting(
        ctx: Context<OpenVoting>,
        year: u32,
        proposal_usdc_bps: u16,
        proposal_sol_bps: u16,
        proposal_btc_bps: u16,
    ) -> Result<()> {
        governance::open_voting_handler(ctx, year, proposal_usdc_bps, proposal_sol_bps, proposal_btc_bps)
    }

    /// Cast a weighted approve/reject vote on the current year's proposal.
    /// Vote weight = voter's FORKIT balance at cast time.
    pub fn cast_vote(ctx: Context<CastVote>, year: u32, approve: bool) -> Result<()> {
        governance::cast_vote_handler(ctx, year, approve)
    }

    /// Tally the vote after Nov 7 23:59 UTC.
    /// If quorum and majority are met, allocations take effect Jan 1.
    pub fn finalize_vote(ctx: Context<FinalizeVote>, year: u32) -> Result<()> {
        governance::finalize_vote_handler(ctx, year)
    }

    /// Propose a new FORKIT mint rate. Starts the 7-day timelock.
    /// Only the program owner may call this.
    pub fn propose_rate_change(ctx: Context<ProposeRateChange>, new_rate: u64) -> Result<()> {
        timelock::propose_rate_change_handler(ctx, new_rate)
    }

    /// Apply the pending mint rate change after the 7-day timelock.
    /// Permissionless once the timelock expires.
    pub fn execute_rate_change(ctx: Context<ExecuteRateChange>) -> Result<()> {
        timelock::execute_rate_change_handler(ctx)
    }

    /// Propose a reserve withdrawal. Any of the 5 multisig signers may propose.
    pub fn request_withdrawal(
        ctx: Context<RequestWithdrawal>,
        amount: u64,
        token_mint: Pubkey,
        recipient: Pubkey,
    ) -> Result<()> {
        multisig::request_withdrawal_handler(ctx, amount, token_mint, recipient)
    }

    /// Add an approval to a pending withdrawal (3-of-5 required).
    pub fn approve_withdrawal(ctx: Context<ApproveWithdrawal>, nonce: u64) -> Result<()> {
        multisig::approve_withdrawal_handler(ctx, nonce)
    }

    /// Execute a withdrawal once 3 approvals are on record.
    /// Permissionless.
    pub fn execute_withdrawal(ctx: Context<ExecuteWithdrawal>, nonce: u64) -> Result<()> {
        multisig::execute_withdrawal_handler(ctx, nonce)
    }
}
