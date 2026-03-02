use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::ForkitError;

#[derive(Accounts)]
pub struct TimeoutRefund<'info> {
    #[account(
        mut,
        seeds = [Order::SEED, &order.order_id.to_le_bytes()],
        bump = order.bump,
    )]
    pub order: Account<'info, Order>,

    #[account(
        mut,
        seeds = [b"escrow_vault", &order.order_id.to_le_bytes()],
        bump,
    )]
    pub escrow_vault: Account<'info, TokenAccount>,

    /// Customer token account to receive refund
    #[account(
        mut,
        constraint = customer_token_account.owner == order.customer,
    )]
    pub customer_token_account: Account<'info, TokenAccount>,

    /// Permissionless crank — anyone can call
    pub cranker: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<TimeoutRefund>) -> Result<()> {
    let order = &ctx.accounts.order;
    let clock = Clock::get()?;

    let timed_out = match order.status {
        OrderStatus::Created | OrderStatus::Preparing => {
            clock.unix_timestamp > order.created_at + PREP_TIMEOUT_SECONDS
        }
        OrderStatus::ReadyForPickup => {
            clock.unix_timestamp > order.created_at + PREP_TIMEOUT_SECONDS + PICKUP_TIMEOUT_SECONDS
        }
        _ => false,
    };

    require!(timed_out, ForkitError::NotTimedOut);

    let escrow_balance = ctx.accounts.escrow_vault.amount;
    let order_id_bytes = order.order_id.to_le_bytes();
    let seeds = &[b"escrow_vault" as &[u8], &order_id_bytes, &[ctx.bumps.escrow_vault]];

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

    let order = &mut ctx.accounts.order;
    order.status = OrderStatus::Refunded;

    emit!(OrderRefunded {
        order_id: order.order_id,
        reason: "Timeout".to_string(),
    });

    Ok(())
}
