use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::ForkitError;

#[derive(Accounts)]
pub struct ResolveDispute<'info> {
    #[account(
        mut,
        seeds = [Order::SEED, &order.order_id.to_le_bytes()],
        bump = order.bump,
    )]
    pub order: Account<'info, Order>,

    #[account(
        seeds = [ProtocolConfig::SEED],
        bump = protocol_config.bump,
        has_one = admin @ ForkitError::Unauthorized,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,

    #[account(
        mut,
        seeds = [ESCROW_VAULT_SEED, &order.order_id.to_le_bytes()],
        bump,
    )]
    pub escrow_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = restaurant_token_account.owner == order.restaurant
            @ ForkitError::Unauthorized,
        constraint = restaurant_token_account.mint == order.token_mint
            @ ForkitError::UnsupportedMint,
    )]
    pub restaurant_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = driver_token_account.owner == order.driver
            @ ForkitError::Unauthorized,
        constraint = driver_token_account.mint == order.token_mint
            @ ForkitError::UnsupportedMint,
    )]
    pub driver_token_account: Account<'info, TokenAccount>,

    /// Arbiter (protocol admin for MVP)
    pub admin: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

/// Resolve a dispute. Three outcomes:
/// - RefundCustomer: set status to Refunded, contributors claim via refund_contributor
/// - PayRestaurantAndDriver: pay them from escrow, set Refunded for remaining contributor claims
/// - Split: pay restaurant+driver half, set Refunded for contributor claims on remainder
pub fn handler(ctx: Context<ResolveDispute>, resolution: DisputeResolution) -> Result<()> {
    let order = &mut ctx.accounts.order;

    require!(
        order.status == OrderStatus::Disputed,
        ForkitError::InvalidOrderStatus
    );

    let order_id_bytes = order.order_id.to_le_bytes();
    let seeds = &[b"escrow_vault" as &[u8], &order_id_bytes, &[ctx.bumps.escrow_vault]];

    match resolution {
        DisputeResolution::RefundCustomer => {
            // All funds stay in escrow — contributors claim via refund_contributor
        }
        DisputeResolution::PayRestaurantAndDriver => {
            require!(
                order.driver != Pubkey::default(),
                ForkitError::DriverNotAssigned
            );

            let restaurant_payout = order.food_amount;
            let driver_payout = order.delivery_amount;

            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.escrow_vault.to_account_info(),
                        to: ctx.accounts.restaurant_token_account.to_account_info(),
                        authority: ctx.accounts.escrow_vault.to_account_info(),
                    },
                    &[seeds],
                ),
                restaurant_payout,
            )?;
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.escrow_vault.to_account_info(),
                        to: ctx.accounts.driver_token_account.to_account_info(),
                        authority: ctx.accounts.escrow_vault.to_account_info(),
                    },
                    &[seeds],
                ),
                driver_payout,
            )?;
            // Remaining excess stays for contributors to claim via refund_contributor
        }
        DisputeResolution::Split => {
            require!(
                order.driver != Pubkey::default(),
                ForkitError::DriverNotAssigned
            );

            let escrow_balance = ctx.accounts.escrow_vault.amount;
            let half = escrow_balance / 2;
            let food_amt = order.food_amount as u128;
            let total = food_amt
                .checked_add(order.delivery_amount as u128)
                .ok_or(ForkitError::ArithmeticOverflow)?;
            let restaurant_share = (half as u128)
                .checked_mul(food_amt)
                .ok_or(ForkitError::ArithmeticOverflow)?
                .checked_div(total)
                .ok_or(ForkitError::ArithmeticOverflow)? as u64;
            // half >= restaurant_share always (food_amt <= total), so this is safe
            let driver_share = half - restaurant_share;

            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.escrow_vault.to_account_info(),
                        to: ctx.accounts.restaurant_token_account.to_account_info(),
                        authority: ctx.accounts.escrow_vault.to_account_info(),
                    },
                    &[seeds],
                ),
                restaurant_share,
            )?;
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.escrow_vault.to_account_info(),
                        to: ctx.accounts.driver_token_account.to_account_info(),
                        authority: ctx.accounts.escrow_vault.to_account_info(),
                    },
                    &[seeds],
                ),
                driver_share,
            )?;
            // Other half stays for contributors to claim via refund_contributor
        }
    }

    order.status = OrderStatus::Refunded;

    emit!(DisputeResolved {
        order_id: order.order_id,
        resolution,
    });

    Ok(())
}
