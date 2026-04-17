use anchor_lang::prelude::*;

#[error_code]
pub enum LoyaltyError {
    #[msg("Only the authorized escrow program may award loyalty points")]
    UnauthorizedCaller,
    #[msg("Point amount must be greater than zero")]
    ZeroPoints,
    #[msg("Insufficient loyalty points balance")]
    InsufficientPoints,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
}
