use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, TokenInterface, TokenAccount, TransferChecked, Mint};
use crate::state::*;
use crate::errors::ForkitError;

/// After successful delivery (Settled), contributors who funded more than their
/// fair share can claim a reimbursement.
///
/// When multiple people contribute to an order, the original payer (who likely
/// fronted the full amount) gets reimbursed proportionally as others chip in.
/// After settlement, each contributor's fair share is:
///   fair_share = escrow_target / contributor_count
/// If a contributor funded more than their fair share, the excess is claimable.
///
/// Example: Order total 100 USDC. Person A paid 100 USDC. Person B contributed 40 USDC.
/// The escrow now has 140 USDC (100 target + 40 excess).
/// After settlement, 100 USDC is distributed to restaurant + driver + treasury.
/// The remaining 40 USDC in the vault is Person A's reimbursement (since Person B
/// effectively covered 40 USDC of the order that Person A had already paid).
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
        seeds = [ESCROW_VAULT_SEED, &order.order_id.to_le_bytes()],
        bump,
    )]
    pub escrow_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(constraint = token_mint.key() == order.token_mint @ ForkitError::UnsupportedMint)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        constraint = contributor_token_account.owner == contributor.key(),
    )]
    pub contributor_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    pub contributor: Signer<'info>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<ClaimDeposit>) -> Result<()> {
    let order = &ctx.accounts.order;
    let contribution = &mut ctx.accounts.contribution;

    // If the escrow was overfunded (total contributions > escrow_target),
    // the excess belongs proportionally to contributors.
    // Each contributor's reimbursement = excess * (their_contribution / total_funded)
    let excess = order.escrow_funded
        .checked_sub(order.escrow_target)
        .unwrap_or(0);

    if excess == 0 {
        // Nothing to reimburse — escrow was funded exactly
        contribution.amount = 0;
        return Ok(());
    }

    let reimbursement = (excess as u128)
        .checked_mul(contribution.amount as u128)
        .ok_or(ForkitError::ArithmeticOverflow)?
        .checked_div(order.escrow_funded as u128)
        .ok_or(ForkitError::ArithmeticOverflow)? as u64;

    if reimbursement == 0 {
        contribution.amount = 0;
        return Ok(());
    }

    let order_id_bytes = order.order_id.to_le_bytes();
    let seeds = &[b"escrow_vault" as &[u8], &order_id_bytes, &[ctx.bumps.escrow_vault]];

    token_interface::transfer_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.escrow_vault.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.contributor_token_account.to_account_info(),
                authority: ctx.accounts.escrow_vault.to_account_info(),
            },
            &[seeds],
        ),
        reimbursement,
        ctx.accounts.token_mint.decimals,
    )?;

    let contributor_key = contribution.contributor;

    // Zero out to prevent double-claim
    contribution.amount = 0;

    emit!(ContributorReimbursed {
        order_id: order.order_id,
        contributor: contributor_key,
        reimbursement,
    });

    Ok(())
}
