use anchor_lang::prelude::*;

pub const CANCEL_WINDOW_SECONDS: i64 = 60;
pub const PREP_TIMEOUT_SECONDS: i64 = 2700; // 45 minutes
pub const PICKUP_TIMEOUT_SECONDS: i64 = 1800; // 30 minutes
pub const DELIVERY_TIMEOUT_SECONDS: i64 = 7200; // 2 hours
pub const DEPOSIT_MULTIPLIER: u64 = 2;
pub const FEE_BASIS_POINTS: u16 = 2; // 0.02%
pub const MAX_ACCEPTED_MINTS: usize = 20;

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
    pub customer: Pubkey,
    pub restaurant: Pubkey,
    pub driver: Pubkey,
    pub token_mint: Pubkey,
    pub food_amount: u64,
    pub delivery_amount: u64,
    pub deposit_amount: u64,
    pub protocol_fee: u64,
    pub code_a_hash: [u8; 32],
    pub code_b_hash: [u8; 32],
    pub status: OrderStatus,
    pub created_at: i64,
    pub cancel_deadline: i64,
    pub pickup_confirmed_at: i64,
    pub delivery_confirmed_at: i64,
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
        8 + // deposit_amount
        8 + // protocol_fee
        32 + // code_a_hash
        32 + // code_b_hash
        1 + // status
        8 + // created_at
        8 + // cancel_deadline
        8 + // pickup_confirmed_at
        8 + // delivery_confirmed_at
        1; // bump
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    Created,
    Preparing,
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
    pub deposit_amount: u64,
    pub protocol_fee: u64,
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
