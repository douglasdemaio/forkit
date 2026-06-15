use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, TokenInterface, TokenAccount, TransferChecked, Mint};
use crate::state::*;
use crate::errors::ForkitError;

#[derive(Accounts)]
pub struct MerchantSettleOrder<'info> {
    #[account(
        mut,
        seeds = [Order::SEED, &order.order_id.to_le_bytes()],
        bump = order.bump,
        has_one = restaurant @ ForkitError::Unauthorized,
    )]
    pub order: Account<'info, Order>,

    #[account(
        mut,
        seeds = [ESCROW_VAULT_SEED, &order.order_id.to_le_bytes()],
        bump,
    )]
    pub escrow_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        seeds = [ProtocolConfig::SEED],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,

    #[account(constraint = token_mint.key() == order.token_mint @ ForkitError::UnsupportedMint)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        constraint = restaurant_token_account.owner == order.restaurant
            @ ForkitError::Unauthorized,
        constraint = restaurant_token_account.mint == order.token_mint
            @ ForkitError::UnsupportedMint,
    )]
    pub restaurant_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = driver_token_account.owner == order.driver
            @ ForkitError::Unauthorized,
        constraint = driver_token_account.mint == order.token_mint
            @ ForkitError::UnsupportedMint,
    )]
    pub driver_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = treasury_token_account.owner == protocol_config.treasury_wallet
            @ ForkitError::Unauthorized,
        constraint = treasury_token_account.mint == order.token_mint
            @ ForkitError::UnsupportedMint,
    )]
    pub treasury_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub restaurant: Signer<'info>,
    pub token_program: Interface<'info, TokenInterface>,
}

/// Merchant-driven settlement — allows the restaurant owner to settle an order
/// on-chain without the customer's signature. Same payout logic as
/// `confirm_delivery` but skips the code-B check and surplus refund (excess
/// funds stay in the vault for contributors to claim via `claim_deposit`).
///
/// Status must be `PickedUp` or `Delivered`.
pub fn handler(ctx: Context<MerchantSettleOrder>) -> Result<()> {
    let order = &mut ctx.accounts.order;

    require!(
        order.status == OrderStatus::PickedUp || order.status == OrderStatus::Delivered,
        ForkitError::InvalidOrderStatus
    );

    require!(
        order.driver != Pubkey::default(),
        ForkitError::DriverNotAssigned
    );

    let clock = Clock::get()?;
    let order_id_bytes = order.order_id.to_le_bytes();
    let seeds = &[b"escrow_vault" as &[u8], &order_id_bytes, &[ctx.bumps.escrow_vault]];

    let food_amt = order.food_amount as u128;
    let delivery_amt = order.delivery_amount as u128;
    let total = food_amt
        .checked_add(delivery_amt)
        .ok_or(ForkitError::ArithmeticOverflow)?;

    let protocol_fee = order.protocol_fee as u128;

    let food_fee = protocol_fee
        .checked_mul(food_amt)
        .ok_or(ForkitError::ArithmeticOverflow)?
        .checked_div(total)
        .ok_or(ForkitError::ArithmeticOverflow)? as u64;

    let delivery_fee = order.protocol_fee
        .checked_sub(food_fee)
        .ok_or(ForkitError::ArithmeticOverflow)?;

    let restaurant_payout = order.food_amount
        .checked_sub(food_fee)
        .ok_or(ForkitError::ArithmeticOverflow)?;
    let driver_payout = order.delivery_amount
        .checked_sub(delivery_fee)
        .ok_or(ForkitError::ArithmeticOverflow)?;

    let decimals = ctx.accounts.token_mint.decimals;

    // Transfer to restaurant
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.escrow_vault.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.restaurant_token_account.to_account_info(),
                authority: ctx.accounts.escrow_vault.to_account_info(),
            },
            &[seeds],
        ),
        restaurant_payout,
        decimals,
    )?;

    // Transfer to driver
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.escrow_vault.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.driver_token_account.to_account_info(),
                authority: ctx.accounts.escrow_vault.to_account_info(),
            },
            &[seeds],
        ),
        driver_payout,
        decimals,
    )?;

    // Transfer protocol fee to treasury
    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.escrow_vault.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.treasury_token_account.to_account_info(),
                authority: ctx.accounts.escrow_vault.to_account_info(),
            },
            &[seeds],
        ),
        order.protocol_fee,
        decimals,
    )?;

    // No surplus refund — the merchant doesn't know the customer's token
    // account. Excess funds remain in the vault for contributors to claim
    // via `claim_deposit`.

    let is_ai_order = order.ai_confidence > 0;

    order.status = OrderStatus::Settled;
    order.delivery_confirmed_at = clock.unix_timestamp;

    emit!(MerchantSettled {
        order_id: order.order_id,
        timestamp: clock.unix_timestamp,
        restaurant_payout,
        driver_payout,
        protocol_fee: order.protocol_fee,
        is_ai_order,
    });

    Ok(())
}
