use anchor_lang::prelude::*;

#[error_code]
pub enum ForkitError {
    #[msg("Cancel window has closed")]
    CancelWindowClosed,
    #[msg("Order is not in the correct status for this operation")]
    InvalidOrderStatus,
    #[msg("Invalid confirmation code")]
    InvalidCode,
    #[msg("Order has expired")]
    OrderExpired,
    #[msg("Insufficient deposit amount")]
    InsufficientDeposit,
    #[msg("Unauthorized: signer is not authorized for this action")]
    Unauthorized,
    #[msg("Unsupported stablecoin mint")]
    UnsupportedMint,
    #[msg("Order has not timed out yet")]
    NotTimedOut,
    #[msg("Delivery timeout has not been reached")]
    DeliveryTimeoutNotReached,
    #[msg("Maximum number of accepted mints reached")]
    MaxMintsReached,
    #[msg("Mint already accepted")]
    MintAlreadyAccepted,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("Invalid fee basis points")]
    InvalidFeeBasisPoints,
    #[msg("Invalid rating value (must be 1-5)")]
    InvalidRating,
}
