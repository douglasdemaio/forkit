use anchor_lang::prelude::*;

#[error_code]
pub enum RegistryError {
    #[msg("Invalid rating value (must be 1-5)")]
    InvalidRating,
    #[msg("A profile cannot rate itself")]
    SelfRatingNotAllowed,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
}
