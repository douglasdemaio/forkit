use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

declare_id!("BnnUntqkUadZ2BsW8j675P9hJQV3aqVcmt4xG4xfeoM8");

/// ForkIt Loyalty Program
///
/// Manages the $FORK-points economy: earning on delivery, tier progression,
/// AI-routing bonuses, and redemption against future orders.
///
/// ## AI Food Delivery Integration
/// When an order is matched and routed by ForkIt's AI (see backend `matching.ts`),
/// the escrow program passes `is_ai_order = true` to earn_points on settlement.
/// This rewards users who trust the AI matcher with a 50% point bonus, driving
/// adoption of the autonomous routing engine.
///
/// ## Tier Benefits
/// | Tier     | Lifetime Pts | Protocol-Fee Discount |
/// |----------|--------------|-----------------------|
/// | None     | 0–499        | 0%                    |
/// | Bronze   | 500–2 499    | 5%                    |
/// | Silver   | 2 500–9 999  | 10%                   |
/// | Gold     | 10 000–49 999| 15%                   |
/// | Platinum | 50 000+      | 20%                   |
#[program]
pub mod forkit_loyalty {
    use super::*;

    /// One-time setup — bind the loyalty program to the escrow authority.
    pub fn initialize_loyalty(
        ctx: Context<InitializeLoyalty>,
        authorized_escrow: Pubkey,
    ) -> Result<()> {
        instructions::initialize_loyalty::handler(ctx, authorized_escrow)
    }

    /// Award points to a recipient. Only callable by `loyalty_config.authorized_escrow`.
    /// Pass `is_ai_order = true` for a 50% bonus when the delivery was AI-routed.
    pub fn earn_points(ctx: Context<EarnPoints>, base_amount: u64, is_ai_order: bool) -> Result<()> {
        instructions::earn_points::handler(ctx, base_amount, is_ai_order)
    }

    /// Burn points from the caller's balance (applied as order discount off-chain).
    pub fn redeem_points(ctx: Context<RedeemPoints>, amount: u64) -> Result<()> {
        instructions::redeem_points::handler(ctx, amount)
    }
}
