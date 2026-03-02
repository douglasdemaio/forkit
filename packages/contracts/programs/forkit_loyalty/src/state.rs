use anchor_lang::prelude::*;

/// Placeholder for future $FORK token loyalty account
#[account]
pub struct LoyaltyAccount {
    pub wallet: Pubkey,
    pub points: u64,
    pub lifetime_earned: u64,
    pub lifetime_redeemed: u64,
    pub bump: u8,
}

impl LoyaltyAccount {
    pub const SEED: &'static [u8] = b"loyalty";
    pub const SPACE: usize = 8 + 32 + 8 + 8 + 8 + 1;
}
