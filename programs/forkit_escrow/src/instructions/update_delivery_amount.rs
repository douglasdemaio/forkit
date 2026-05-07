use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::ForkitError;

/// Restaurant lowers the order's `delivery_amount` after accepting a driver's bid
/// that's below the posted fee. The surplus stays in the escrow vault and is
/// refunded to the customer at `confirm_delivery`.
///
/// Constraints:
///   - Order must be in `Funded` status (driver hasn't called `accept_order` yet).
///   - No driver assigned yet (`order.driver == Pubkey::default()`).
///   - `new_delivery_amount` must be > 0 and <= current `delivery_amount`.
///   - Signer must match `order.restaurant`.
///
/// Side effects:
///   - `order.delivery_amount` is updated.
///   - `order.protocol_fee` is recomputed against the new (food + delivery) total
///     using the protocol's current fee basis points, so the fee scales with the
///     reduced delivery price (otherwise restaurants accepting low bids would
///     pay an unintentionally higher effective fee rate).
#[derive(Accounts)]
pub struct UpdateDeliveryAmount<'info> {
    #[account(
        mut,
        seeds = [Order::SEED, &order.order_id.to_le_bytes()],
        bump = order.bump,
        has_one = restaurant @ ForkitError::Unauthorized,
    )]
    pub order: Account<'info, Order>,

    #[account(
        seeds = [ProtocolConfig::SEED],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,

    pub restaurant: Signer<'info>,
}

pub fn handler(ctx: Context<UpdateDeliveryAmount>, new_delivery_amount: u64) -> Result<()> {
    let order = &mut ctx.accounts.order;

    require!(
        order.status == OrderStatus::Funded,
        ForkitError::InvalidOrderStatus
    );
    require!(
        order.driver == Pubkey::default(),
        ForkitError::DriverAlreadyAssigned
    );
    require!(new_delivery_amount > 0, ForkitError::InvalidDeliveryAmount);
    require!(
        new_delivery_amount <= order.delivery_amount,
        ForkitError::CannotIncreaseDelivery
    );

    // Recompute protocol fee against the new (food + new_delivery) total so
    // the protocol's basis-point rate stays accurate regardless of bid amount.
    let new_total = order
        .food_amount
        .checked_add(new_delivery_amount)
        .ok_or(ForkitError::ArithmeticOverflow)?;
    let new_protocol_fee = (new_total as u128)
        .checked_mul(ctx.accounts.protocol_config.fee_basis_points as u128)
        .ok_or(ForkitError::ArithmeticOverflow)?
        .checked_div(10_000)
        .ok_or(ForkitError::ArithmeticOverflow)? as u64;

    order.delivery_amount = new_delivery_amount;
    order.protocol_fee = new_protocol_fee;

    emit!(DeliveryAmountUpdated {
        order_id: order.order_id,
        new_delivery_amount,
        new_protocol_fee,
    });

    Ok(())
}

#[event]
pub struct DeliveryAmountUpdated {
    pub order_id: u64,
    pub new_delivery_amount: u64,
    pub new_protocol_fee: u64,
}
