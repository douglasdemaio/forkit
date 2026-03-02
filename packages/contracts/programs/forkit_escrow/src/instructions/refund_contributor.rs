use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::ForkitError;

/// Refunds a specific contributor's share from escrow.
/// Called per-contributor after cancel or timeout sets order to Cancelled/Refunded.
/// Permissionless — anyone can crank refunds for any contributor.
#[derive(Accounts)]
pub struct RefundContributor<'info> {
    #[account(
        seeds = [Order::SEED, &order.order_id.to_le_bytes()],
        bump = order.bump,
        constraint = order.status == OrderStatus::Cancelled || order.status == OrderStatus::Refunded @ ForkitError::InvalidOrderStatus,
    )]
    pub order: Account<'info, Order>,

    #[account(
        mut,
        seeds = [Contribution::SEED, &order.order_id.to_le_bytes(), contribution.contributor.as_ref()],
        bump = contribution.bump,
        constraint = contribution.amount > 0 @ ForkitError::NoContribution,
    )]
    pub contribution: Account<'info, Contribution>,

    #[account(
        mut,
        seeds = [b"escrow_vault", &order.order_id.to_le_bytes()],
        bump,
    )]
    pub escrow_vault: Account<'info, TokenAccount>,

    /// Contributor's token account to receive refund
    #[account(
        mut,
        constraint = contributor_token_account.owner == contribution.contributor,
    )]
    pub contributor_token_account: Account<'info, TokenAccount>,

    /// Permissionless crank
    pub cranker: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<RefundContributor>) -> Result<()> {
    let contribution = &mut ctx.accounts.contribution;
    let refund_amount = contribution.amount;
    let contributor = contribution.contributor;
    let order_id = contribution.order_id;

    let order_id_bytes = ctx.accounts.order.order_id.to_le_bytes();
    let seeds = &[b"escrow_vault" as &[u8], &order_id_bytes, &[ctx.bumps.escrow_vault]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_vault.to_account_info(),
                to: ctx.accounts.contributor_token_account.to_account_info(),
                authority: ctx.accounts.escrow_vault.to_account_info(),
            },
            &[seeds],
        ),
        refund_amount,
    )?;

    // Zero out contribution to prevent double-refund
    contribution.amount = 0;

    emit!(ContributorRefunded {
        order_id,
        contributor,
        amount: refund_amount,
    });

    Ok(())
}
