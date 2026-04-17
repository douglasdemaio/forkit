use anchor_lang::prelude::*;

/// Tier thresholds (lifetime points earned)
pub const TIER_BRONZE_THRESHOLD: u64 = 500;
pub const TIER_SILVER_THRESHOLD: u64 = 2_500;
pub const TIER_GOLD_THRESHOLD: u64 = 10_000;
pub const TIER_PLATINUM_THRESHOLD: u64 = 50_000;

/// Points awarded per order (in basis — caller scales by order size)
pub const POINTS_PER_ORDER_BPS: u64 = 100; // 1% of order value in points

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Tier {
    None,
    Bronze,
    Silver,
    Gold,
    Platinum,
}

impl Tier {
    pub fn from_lifetime(lifetime_earned: u64) -> Self {
        if lifetime_earned >= TIER_PLATINUM_THRESHOLD {
            Tier::Platinum
        } else if lifetime_earned >= TIER_GOLD_THRESHOLD {
            Tier::Gold
        } else if lifetime_earned >= TIER_SILVER_THRESHOLD {
            Tier::Silver
        } else if lifetime_earned >= TIER_BRONZE_THRESHOLD {
            Tier::Bronze
        } else {
            Tier::None
        }
    }

    /// Discount in basis points applied to protocol fee for this tier
    pub fn discount_bps(&self) -> u16 {
        match self {
            Tier::None => 0,
            Tier::Bronze => 500,    // 5% off protocol fee
            Tier::Silver => 1000,   // 10% off
            Tier::Gold => 1500,     // 15% off
            Tier::Platinum => 2000, // 20% off
        }
    }
}

/// Global loyalty program configuration.
/// Controls who is authorized to award points (only the escrow program).
#[account]
pub struct LoyaltyConfig {
    pub admin: Pubkey,
    /// The escrow program's derived authority that may call earn_points.
    pub authorized_escrow: Pubkey,
    /// Total lifetime points awarded across all users (protocol stat).
    pub total_points_issued: u64,
    pub bump: u8,
}

impl LoyaltyConfig {
    pub const SEED: &'static [u8] = b"loyalty_config";
    pub const SPACE: usize = 8 + 32 + 32 + 8 + 1;
}

/// Per-user loyalty account.
#[account]
pub struct LoyaltyAccount {
    pub wallet: Pubkey,
    pub points: u64,
    pub lifetime_earned: u64,
    pub lifetime_redeemed: u64,
    /// Current tier (recomputed on each earn)
    pub tier: Tier,
    /// Timestamp of last points award (for analytics / future cooldowns)
    pub last_earned_at: i64,
    /// Orders completed via AI-routed delivery (qualifies for bonus points)
    pub ai_orders_count: u64,
    pub bump: u8,
}

impl LoyaltyAccount {
    pub const SEED: &'static [u8] = b"loyalty";
    pub const SPACE: usize = 8 + // discriminator
        32 + // wallet
        8 +  // points
        8 +  // lifetime_earned
        8 +  // lifetime_redeemed
        1 +  // tier (enum u8)
        8 +  // last_earned_at
        8 +  // ai_orders_count
        1;   // bump

    /// Recalculates and sets tier from current lifetime_earned. Returns true if tier changed.
    pub fn update_tier(&mut self) -> bool {
        let new_tier = Tier::from_lifetime(self.lifetime_earned);
        if new_tier != self.tier {
            self.tier = new_tier;
            return true;
        }
        false
    }
}

// Events

#[event]
pub struct PointsEarned {
    pub wallet: Pubkey,
    pub amount: u64,
    pub is_ai_order: bool,
    pub new_balance: u64,
    pub lifetime_earned: u64,
}

#[event]
pub struct PointsRedeemed {
    pub wallet: Pubkey,
    pub amount: u64,
    pub new_balance: u64,
}

#[event]
pub struct TierUpgraded {
    pub wallet: Pubkey,
    pub new_tier: Tier,
    pub lifetime_earned: u64,
}
