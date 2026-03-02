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
        seeds = [b"escrow_vault", &order.order_id.to_le_bytes()],
        bump,
    )]
    pub escrow_vault: Account<'info, TokenAccount>,

    #[account(mut, constraint = customer_token_account.owner == order.customer)]
    pub customer_token_account: Account<'info, TokenAccount>,

    #[account(mut, constraint = restaurant_token_account.owner == order.restaurant)]
    pub restaurant_token_account: Account<'info, TokenAccount>,

    #[account(mut, constraint = driver_token_account.owner == order.driver)]
    pub driver_token_account: Account<'info, TokenAccount>,

    /// Arbiter (protocol admin for MVP)
    pub admin: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<ResolveDispute>, resolution: DisputeResolution) -> Result<()> {
    let order = &mut ctx.accounts.order;

    require!(
        order.status == OrderStatus::Disputed,
        ForkitError::InvalidOrderStatus
    );

    let escrow_balance = ctx.accounts.escrow_vault.amount;
    let order_id_bytes = order.order_id.to_le_bytes();
    let seeds = &[b"escrow_vault" as &[u8], &order_id_bytes, &[ctx.bumps.escrow_vault]];

    match resolution {
        DisputeResolution::RefundCustomer => {
            // Full refund to customer
            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.escrow_vault.to_account_info(),
                        to: ctx.accounts.customer_token_account.to_account_info(),
                        authority: ctx.accounts.escrow_vault.to_account_info(),
                    },
                    &[seeds],
                ),
                escrow_balance,
            )?;
        }
        DisputeResolution::PayRestaurantAndDriver => {
            // Pay restaurant and driver, return deposit minus penalty
            let restaurant_payout = order.food_amount;
            let driver_payout = order.delivery_amount;
            let customer_refund = escrow_balance - restaurant_payout - driver_payout;

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
            if customer_refund > 0 {
                token::transfer(
                    CpiContext::new_with_signer(
                        ctx.accounts.token_program.to_account_info(),
                        Transfer {
                            from: ctx.accounts.escrow_vault.to_account_info(),
                            to: ctx.accounts.customer_token_account.to_account_info(),
                            authority: ctx.accounts.escrow_vault.to_account_info(),
                        },
                        &[seeds],
                    ),
                    customer_refund,
                )?;
            }
        }
        DisputeResolution::Split => {
            // 50/50 split between customer and restaurant+driver
            let half = escrow_balance / 2;
            let other_half = escrow_balance - half;

            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.escrow_vault.to_account_info(),
                        to: ctx.accounts.customer_token_account.to_account_info(),
                        authority: ctx.accounts.escrow_vault.to_account_info(),
                    },
                    &[seeds],
                ),
                half,
            )?;
            // Split remainder between restaurant and driver proportionally
            let total = order.food_amount + order.delivery_amount;
            let restaurant_share = other_half * order.food_amount / total;
            let driver_share = other_half - restaurant_share;

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
        }
    }

    order.status = OrderStatus::Refunded;

    emit!(DisputeResolved {
        order_id: order.order_id,
        resolution,
    });

    Ok(())
}
