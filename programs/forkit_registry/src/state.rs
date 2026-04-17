use anchor_lang::prelude::*;

pub const MAX_METADATA_URI_LEN: usize = 200;
pub const TRUST_DECAY_THRESHOLD_DAYS: i64 = 30;
pub const SECONDS_PER_DAY: i64 = 86400;

#[account]
pub struct Profile {
    pub wallet: Pubkey,
    pub role: Role,
    pub trust_score: u32,        // 0-10000 (0.00-100.00)
    pub completed_orders: u64,
    pub total_ratings: u64,
    pub sum_ratings: u64,
    pub disputes_filed: u64,
    pub disputes_lost: u64,
    pub loyalty_points: u64,
    pub registered_at: i64,
    pub last_active_at: i64,
    pub metadata_uri: String,
    pub is_active: bool,
    pub bump: u8,
}

impl Profile {
    pub const SEED: &'static [u8] = b"profile";

    pub const SPACE: usize = 8 + // discriminator
        32 + // wallet
        1 + // role
        4 + // trust_score
        8 + // completed_orders
        8 + // total_ratings
        8 + // sum_ratings
        8 + // disputes_filed
        8 + // disputes_lost
        8 + // loyalty_points
        8 + // registered_at
        8 + // last_active_at
        4 + MAX_METADATA_URI_LEN + // metadata_uri
        1 + // is_active
        1; // bump

    pub fn recalculate_trust(&mut self, current_timestamp: i64) {
        if self.total_ratings == 0 {
            return;
        }

        // Base score: map 1-5 star average to 2000-10000
        let avg_rating_x1000 = (self.sum_ratings * 1000) / self.total_ratings;
        let base_score = (avg_rating_x1000 * 2000) / 5000;

        // Dispute penalty
        let score = if self.completed_orders > 0 && self.disputes_lost > 0 {
            let penalty_ratio = (self.disputes_lost * 10000) / self.completed_orders;
            base_score * (10000 - penalty_ratio.min(5000)) / 10000
        } else {
            base_score
        };

        // Decay for inactivity
        let days_inactive = (current_timestamp - self.last_active_at) / SECONDS_PER_DAY;
        let final_score = if days_inactive > TRUST_DECAY_THRESHOLD_DAYS {
            let decay_days = days_inactive - TRUST_DECAY_THRESHOLD_DAYS;
            let decay_factor = 10000u64.saturating_sub(decay_days as u64 * 100); // 1% per day
            score * decay_factor / 10000
        } else {
            score
        };

        self.trust_score = final_score.min(10000) as u32;
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Restaurant,
    Driver,
    Customer,
}

#[event]
pub struct ProfileRegistered {
    pub wallet: Pubkey,
    pub role: Role,
}

#[event]
pub struct ProfileRated {
    pub target: Pubkey,
    pub rating: u8,
    pub new_trust_score: u32,
}
