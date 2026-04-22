use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod errors;

use instructions::*;

declare_id!("CNUWqYhXPXszPuB8psqG2VSnwCXf1MWzT4Pztp4y8fgj");

#[program]
pub mod forkit_escrow {
    use super::*;

    pub fn initialize_protocol(
        ctx: Context<InitializeProtocol>,
        fee_basis_points: u16,
    ) -> Result<()> {
        instructions::initialize_protocol::handler(ctx, fee_basis_points)
    }

    pub fn update_protocol_config(
        ctx: Context<UpdateProtocolConfig>,
        new_fee_basis_points: Option<u16>,
        new_treasury: Option<Pubkey>,
    ) -> Result<()> {
        instructions::update_protocol_config::handler(ctx, new_fee_basis_points, new_treasury)
    }

    pub fn add_accepted_mint(ctx: Context<AddAcceptedMint>) -> Result<()> {
        instructions::add_accepted_mint::handler(ctx)
    }

    /// Create an order. The creator can optionally contribute an initial amount.
    /// If initial_contribution covers the full escrow_target, the order moves
    /// directly to Funded status. Otherwise, others can chip in via contribute_to_order.
    ///
    /// `estimated_delivery_time` and `ai_confidence` are provided by the backend AI
    /// routing engine and stored on-chain for transparency and driver prioritisation.
    /// Pass 0 for both if no AI routing is applied.
    ///
    /// If a `surge_config` account is passed and `active`, the delivery fee is scaled
    /// by the current surge multiplier before being locked into the escrow.
    pub fn create_order(
        ctx: Context<CreateOrder>,
        order_id: u64,
        food_amount: u64,
        delivery_amount: u64,
        initial_contribution: u64,
        code_a_hash: [u8; 32],
        code_b_hash: [u8; 32],
        estimated_delivery_time: i64,
        ai_confidence: u8,
        requested_delivery_time: i64,
        requested_pickup_time: i64,
    ) -> Result<()> {
        instructions::create_order::handler(
            ctx, order_id, food_amount, delivery_amount,
            initial_contribution, code_a_hash, code_b_hash,
            estimated_delivery_time, ai_confidence,
            requested_delivery_time, requested_pickup_time,
        )
    }

    /// Anyone can contribute funds to an open order's escrow.
    /// When total contributions reach the escrow_target, order moves to Funded.
    pub fn contribute_to_order(ctx: Context<ContributeToOrder>, amount: u64) -> Result<()> {
        instructions::contribute_to_order::handler(ctx, amount)
    }

    pub fn accept_order(ctx: Context<AcceptOrder>) -> Result<()> {
        instructions::accept_order::handler(ctx)
    }

    pub fn cancel_order(ctx: Context<CancelOrder>) -> Result<()> {
        instructions::cancel_order::handler(ctx)
    }

    pub fn mark_ready_for_pickup(ctx: Context<MarkReadyForPickup>) -> Result<()> {
        instructions::mark_ready_for_pickup::handler(ctx)
    }

    pub fn confirm_pickup(ctx: Context<ConfirmPickup>, code_a: String) -> Result<()> {
        instructions::confirm_pickup::handler(ctx, code_a)
    }

    pub fn confirm_delivery(ctx: Context<ConfirmDelivery>, code_b: String) -> Result<()> {
        instructions::confirm_delivery::handler(ctx, code_b)
    }

    /// After settlement, contributors claim proportional reimbursement of excess funds.
    pub fn claim_deposit(ctx: Context<ClaimDeposit>) -> Result<()> {
        instructions::claim_deposit::handler(ctx)
    }

    /// After cancel or timeout, refund a specific contributor's full amount.
    /// Permissionless — anyone can crank this for any contributor.
    pub fn refund_contributor(ctx: Context<RefundContributor>) -> Result<()> {
        instructions::refund_contributor::handler(ctx)
    }

    pub fn timeout_refund(ctx: Context<TimeoutRefund>) -> Result<()> {
        instructions::timeout_refund::handler(ctx)
    }

    pub fn open_dispute(ctx: Context<OpenDispute>) -> Result<()> {
        instructions::open_dispute::handler(ctx)
    }

    pub fn resolve_dispute(
        ctx: Context<ResolveDispute>,
        resolution: state::DisputeResolution,
    ) -> Result<()> {
        instructions::resolve_dispute::handler(ctx, resolution)
    }

    /// Update the AI surge-pricing multiplier. Only callable by the protocol admin.
    ///
    /// The ForkIt AI pricing service calls this instruction whenever its demand
    /// forecasting model detects a supply/demand imbalance — e.g. a lunch rush,
    /// adverse weather, or a local event driving order spikes.  Setting `active = false`
    /// disables surge without clearing the stored multiplier value.
    pub fn set_surge_pricing(
        ctx: Context<SetSurgePricing>,
        multiplier_bps: u16,
        active: bool,
    ) -> Result<()> {
        instructions::set_surge_pricing::handler(ctx, multiplier_bps, active)
    }
}
