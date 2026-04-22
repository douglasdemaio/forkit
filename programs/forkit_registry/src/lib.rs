use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("EM1FgSzfS3F7cCYJWhUaqqPAK7ijZYpYRx7pzYkuyExz");

#[program]
pub mod forkit_registry {
    use super::*;

    pub fn register(ctx: Context<Register>, role: state::Role, metadata_uri: String) -> Result<()> {
        instructions::register::handler(ctx, role, metadata_uri)
    }

    pub fn update_metadata(ctx: Context<UpdateMetadata>, new_uri: String) -> Result<()> {
        instructions::update_metadata::handler(ctx, new_uri)
    }

    /// Update the payout wallet for a profile. Only the profile owner can call this.
    /// Emits a PayoutWalletChanged event for on-chain audit trail.
    pub fn update_payout_wallet(
        ctx: Context<UpdatePayoutWallet>,
        new_payout_wallet: Pubkey,
    ) -> Result<()> {
        instructions::update_payout_wallet::handler(ctx, new_payout_wallet)
    }

    pub fn rate_counterparty(
        ctx: Context<RateCounterparty>,
        _order_id: u64,
        rating: u8,
    ) -> Result<()> {
        instructions::rate_counterparty::handler(ctx, rating)
    }

    pub fn update_loyalty_points(
        ctx: Context<UpdateLoyaltyPoints>,
        points_delta: i64,
    ) -> Result<()> {
        instructions::update_loyalty_points::handler(ctx, points_delta)
    }
}
