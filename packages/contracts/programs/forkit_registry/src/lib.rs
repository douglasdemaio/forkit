use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("ForkReg111111111111111111111111111111111");

#[program]
pub mod forkit_registry {
    use super::*;

    pub fn register(ctx: Context<Register>, role: state::Role, metadata_uri: String) -> Result<()> {
        instructions::register::handler(ctx, role, metadata_uri)
    }

    pub fn update_metadata(ctx: Context<UpdateMetadata>, new_uri: String) -> Result<()> {
        instructions::update_metadata::handler(ctx, new_uri)
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
