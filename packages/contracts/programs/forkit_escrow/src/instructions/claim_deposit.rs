use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::errors::ForkitError;

/// After successful delivery (Settled), each contributor claims their
/// proportional share of the deposit that was returned.
///
/// deposit_share = (contribution.amount / order.escrow_funded) * order.deposit_amount
///
/// Example: Order total 20 USDC, deposit 40 USDC, escrow_target 60 USDC.
/// Person A contributed 42 USDC (70%), Person B contributed 18 USDC (30%).
/// Person A's deposit share = 40 * 42/60 = 28 USDC
/// Person B's deposit share = 40 * 18/60 = 12 USDC
#[derive(Accounts)]
pub struct ClaimDeposit<'info> {
    #[account(
        seeds = [Order::SEED, &order.order_id.to_le_bytes()],
        bump = order.bump,
        constraint = order.status == OrderStatus::Settled @ ForkitError::InvalidOrderStatus,
    )]
    pub order: Account<'info, Order>,

    #[account(
        mut,
        seeds = [Contribution::SEED, &order.order_id.to_le_bytes(), contributor.key().as_ref()],
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

    #[account(
        mut,
        constraint = contributor_token_account.owner == contributor.key(),
    )]
    pub contributor_token_account: Account<'info, TokenAccount>,

    pub contributor: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<ClaimDeposit>) -> Result<()> {
    let order = &ctx.accounts.order;
    let contribution = &mut ctx.accounts.contribution;

    // Calculate proportional deposit share
    // deposit_share = deposit_amount * contribution / escrow_funded
    let deposit_share = (order.deposit_amount as u128)
        .checked_mul(contribution.amount as u128)
        .ok_or(ForkitError::ArithmeticOverflow)?
        .checked_div(order.escrow_funded as u128)
        .ok_or(ForkitError::ArithmeticOverflow)? as u64;

    let order_id_bytes = order.order_id.to_le_bytes();
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
        deposit_share,
    )?;

    let contributor_key = contribution.contributor;

    // Zero out to prevent double-claim
    contribution.amount = 0;

    emit!(DepositReturned {
        order_id: order.order_id,
        contributor: contributor_key,
        deposit_share,
    });

    Ok(())
}
