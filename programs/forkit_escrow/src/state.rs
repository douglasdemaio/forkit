use anchor_lang::prelude::*;

pub const CANCEL_WINDOW_SECONDS: i64 = 60;
pub const PREP_TIMEOUT_SECONDS: i64 = 2700; // 45 minutes
pub const PICKUP_TIMEOUT_SECONDS: i64 = 2700; // 45 minutes
pub const DELIVERY_TIMEOUT_SECONDS: i64 = 10800; // 3 hours
pub const FUNDING_TIMEOUT_SECONDS: i64 = 900; // 15 minutes to fully fund
pub const FEE_BASIS_POINTS: u16 = 2; // 0.02%
pub const MAX_ACCEPTED_MINTS: usize = 20;
pub const MAX_CONTRIBUTORS: usize = 10;
/// Typed as `&[u8]` (not `&[u8; N]`) so it can be used in Anchor seed constraint arrays
/// alongside other `&[u8]` seeds without type-size conflicts.
pub const ESCROW_VAULT_SEED: &[u8] = b"escrow_vault";
/// Max surge multiplier: 3× (30 000 basis points on top of 10 000 base)
pub const MAX_SURGE_MULTIPLIER_BPS: u16 = 30_000;

/// Dynamic surge-pricing configuration, updated by AI demand prediction.
///
/// When `active`, the delivery fee passed into `create_order` is multiplied by
/// `(10_000 + multiplier_bps) / 10_000` before being stored on the order.
/// This allows the off-chain AI pricing engine to reflect real-time demand
/// without requiring a program upgrade.
#[account]
pub struct SurgeConfig {
    pub admin: Pubkey,
    /// Additional basis points on top of the base delivery fee.
    /// e.g. 5_000 = +50% surge. Capped at MAX_SURGE_MULTIPLIER_BPS.
    pub multiplier_bps: u16,
    /// Whether surge pricing is currently active.
    pub active: bool,
    /// Unix timestamp of the last update (for audit / transparency).
    pub updated_at: i64,
    pub bump: u8,
}

impl SurgeConfig {
    pub const SEED: &'static [u8] = b"surge_config";
    pub const SPACE: usize = 8 + 32 + 2 + 1 + 8 + 1;

    /// Returns the effective delivery amount after applying surge.
    pub fn apply_surge(&self, base_delivery_amount: u64) -> Option<u64> {
        if !self.active || self.multiplier_bps == 0 {
            return Some(base_delivery_amount);
        }
        let multiplied = base_delivery_amount
            .checked_mul(10_000 + self.multiplier_bps as u64)?
            .checked_div(10_000)?;
        Some(multiplied)
    }
}

#[account]
pub struct ProtocolConfig {
    pub admin: Pubkey,
    pub treasury_wallet: Pubkey,
    pub fee_basis_points: u16,
    pub accepted_mints: Vec<Pubkey>,
    pub bump: u8,
}

impl ProtocolConfig {
    pub const SEED: &'static [u8] = b"protocol_config";

    pub fn space(num_mints: usize) -> usize {
        8 + // discriminator
        32 + // admin
        32 + // treasury_wallet
        2 + // fee_basis_points
        4 + (32 * num_mints) + // accepted_mints vec
        1 // bump
    }

    pub fn is_mint_accepted(&self, mint: &Pubkey) -> bool {
        self.accepted_mints.contains(mint)
    }
}

#[account]
pub struct Order {
    pub order_id: u64,
    pub customer: Pubkey,         // order creator — receives delivery codes
    pub restaurant: Pubkey,
    pub driver: Pubkey,
    pub token_mint: Pubkey,
    pub food_amount: u64,
    pub delivery_amount: u64,
    pub protocol_fee: u64,
    pub escrow_target: u64,       // total needed in escrow (food+delivery)
    pub escrow_funded: u64,       // total funded so far
    pub contributor_count: u8,    // number of contributors
    pub code_a_hash: [u8; 32],
    pub code_b_hash: [u8; 32],
    pub status: OrderStatus,
    pub created_at: i64,
    pub cancel_deadline: i64,
    pub pickup_confirmed_at: i64,
    pub delivery_confirmed_at: i64,
    /// Unix timestamp the AI routing engine predicts the order will be delivered.
    /// Set at order creation by the backend AI model; 0 if not provided.
    pub estimated_delivery_time: i64,
    /// Customer-requested delivery time (Unix timestamp). 0 = ASAP.
    /// Allows customers to schedule a preferred delivery window.
    pub requested_delivery_time: i64,
    /// Customer-requested pickup time (Unix timestamp). 0 = ASAP.
    /// Allows customers to schedule a preferred pickup window.
    pub requested_pickup_time: i64,
    /// AI routing confidence score (0–100). Used by drivers to prioritise
    /// high-confidence routes. 0 means no AI routing was applied.
    pub ai_confidence: u8,
    pub bump: u8,
}

impl Order {
    pub const SEED: &'static [u8] = b"order";

    pub const SPACE: usize = 8 + // discriminator
        8 + // order_id
        32 + // customer
        32 + // restaurant
        32 + // driver
        32 + // token_mint
        8 + // food_amount
        8 + // delivery_amount
        8 + // protocol_fee
        8 + // escrow_target
        8 + // escrow_funded
        1 + // contributor_count
        32 + // code_a_hash
        32 + // code_b_hash
        1 + // status
        8 + // created_at
        8 + // cancel_deadline
        8 + // pickup_confirmed_at
        8 + // delivery_confirmed_at
        8 + // estimated_delivery_time
        8 + // requested_delivery_time
        8 + // requested_pickup_time
        1 + // ai_confidence
        1; // bump

    pub fn is_fully_funded(&self) -> bool {
        self.escrow_funded >= self.escrow_target
    }
}

/// Tracks each contributor's share in an order.
/// Anyone can contribute — the order creator, friends, strangers.
/// On refund/cancel, each contributor gets back proportional to their contribution.
#[account]
pub struct Contribution {
    pub order_id: u64,
    pub contributor: Pubkey,
    pub amount: u64,       // total tokens contributed
    pub bump: u8,
}

impl Contribution {
    pub const SEED: &'static [u8] = b"contribution";

    pub const SPACE: usize = 8 + // discriminator
        8 + // order_id
        32 + // contributor
        8 + // amount
        1; // bump
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Created,       // order placed, awaiting full funding
    Funded,        // escrow fully funded, ready for restaurant
    Preparing,     // restaurant accepted
    ReadyForPickup,
    PickedUp,
    Delivered,
    Settled,
    Disputed,
    Cancelled,
    Refunded,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum DisputeResolution {
    RefundCustomer,
    PayRestaurantAndDriver,
    Split,
}

// Events
#[event]
pub struct OrderCreated {
    pub order_id: u64,
    pub customer: Pubkey,
    pub restaurant: Pubkey,
    pub token_mint: Pubkey,
    pub food_amount: u64,
    pub delivery_amount: u64,
    pub escrow_target: u64,
    pub protocol_fee: u64,
    pub requested_delivery_time: i64,
    pub requested_pickup_time: i64,
}

#[event]
pub struct OrderFunded {
    pub order_id: u64,
    pub total_funded: u64,
    pub contributor_count: u8,
}

#[event]
pub struct ContributionMade {
    pub order_id: u64,
    pub contributor: Pubkey,
    pub amount: u64,
    pub total_funded: u64,
    pub escrow_target: u64,
    pub fully_funded: bool,
}

#[event]
pub struct OrderAccepted {
    pub order_id: u64,
    pub driver: Pubkey,
}

#[event]
pub struct OrderCancelled {
    pub order_id: u64,
}

#[event]
pub struct OrderReadyForPickup {
    pub order_id: u64,
}

#[event]
pub struct PickupConfirmed {
    pub order_id: u64,
    pub driver: Pubkey,
    pub timestamp: i64,
}

#[event]
pub struct DeliveryConfirmed {
    pub order_id: u64,
    pub timestamp: i64,
    pub restaurant_payout: u64,
    pub driver_payout: u64,
    pub protocol_fee: u64,
    /// Loyalty points to be awarded to the customer (1% of total order value).
    /// The backend listens for this event and calls `earn_points` on the loyalty program.
    pub loyalty_points_for_customer: u64,
    /// Whether this order was AI-routed (ai_confidence > 0). If true, the loyalty
    /// program will apply its 50% AI bonus on top of loyalty_points_for_customer.
    pub is_ai_order: bool,
}

#[event]
pub struct ContributorRefunded {
    pub order_id: u64,
    pub contributor: Pubkey,
    pub amount: u64,
}

#[event]
pub struct ContributorReimbursed {
    pub order_id: u64,
    pub contributor: Pubkey,
    pub reimbursement: u64,
}

#[event]
pub struct OrderRefunded {
    pub order_id: u64,
    pub reason: String,
}

#[event]
pub struct DisputeOpened {
    pub order_id: u64,
    pub customer: Pubkey,
}

#[event]
pub struct DisputeResolved {
    pub order_id: u64,
    pub resolution: DisputeResolution,
}

#[event]
pub struct MerchantSettled {
    pub order_id: u64,
    pub timestamp: i64,
    pub restaurant_payout: u64,
    pub driver_payout: u64,
    pub protocol_fee: u64,
    pub is_ai_order: bool,
}

#[event]
pub struct SurgeUpdated {
    pub multiplier_bps: u16,
    pub active: bool,
    pub updated_at: i64,
}

#[event]
pub struct OrderCreatedWithAI {
    pub order_id: u64,
    pub estimated_delivery_time: i64,
    pub ai_confidence: u8,
    pub surge_applied: bool,
    pub effective_delivery_amount: u64,
}
