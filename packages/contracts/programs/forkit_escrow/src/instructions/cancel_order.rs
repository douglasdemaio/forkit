use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::ForkitError;

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    #[account(
        mut,
        seeds = [Order::SEED, &order.order_id.to_le_bytes()],
        bump = order.bump,
        has_one = customer @ ForkitError::Unauthorized,
    )]
    pub order: Account<'info, Order>,

    #[account(
        mut,
        seeds = [b"escrow_vault", &order.order_id.to_le_bytes()],
        bump,
    )]
    pub escrow_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = customer_token_account.owner == customer.key(),
    )]
    pub customer_token_account: Account<'info, TokenAccount>,

    pub customer: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<CancelOrder>) -> Result<()> {
    let order = &mut ctx.accounts.order;
    let clock = Clock::get()?;

    require!(
        order.status == OrderStatus::Created || order.status == OrderStatus::Preparing,
        ForkitError::InvalidOrderStatus
    );
    require!(
        clock.unix_timestamp < order.cancel_deadline,
        ForkitError::CancelWindowClosed
    );

    let escrow_balance = ctx.accounts.escrow_vault.amount;
    let order_id_bytes = order.order_id.to_le_bytes();
    let seeds = &[b"escrow_vault" as &[u8], &order_id_bytes, &[ctx.bumps.escrow_vault]];

    // Return all funds to customer
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

    order.status = OrderStatus::Cancelled;

    emit!(OrderCancelled {
        order_id: order.order_id,
    });

    Ok(())
}
