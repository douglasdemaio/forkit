use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;
pub mod errors;

use instructions::*;

declare_id!("ForkEscrow1111111111111111111111111111111");

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
    pub fn create_order(
        ctx: Context<CreateOrder>,
        order_id: u64,
        food_amount: u64,
        delivery_amount: u64,
        initial_contribution: u64,
        code_a_hash: [u8; 32],
        code_b_hash: [u8; 32],
    ) -> Result<()> {
        instructions::create_order::handler(
            ctx, order_id, food_amount, delivery_amount,
            initial_contribution, code_a_hash, code_b_hash,
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

    /// After settlement, each contributor claims their proportional deposit share.
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
}
