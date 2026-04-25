use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, TokenInterface, TokenAccount, TransferChecked, Mint};
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
        seeds = [ESCROW_VAULT_SEED, &order.order_id.to_le_bytes()],
        bump,
    )]
    pub escrow_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(constraint = token_mint.key() == order.token_mint @ ForkitError::UnsupportedMint)]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    /// Contributor's token account to receive refund
    #[account(
        mut,
        constraint = contributor_token_account.owner == contribution.contributor,
    )]
    pub contributor_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    /// Permissionless crank
    pub cranker: Signer<'info>,
    pub token_program: Interface<'info, TokenInterface>,
}

pub fn handler(ctx: Context<RefundContributor>) -> Result<()> {
    let contribution = &mut ctx.accounts.contribution;
    let refund_amount = contribution.amount;
    let contributor = contribution.contributor;
    let order_id = contribution.order_id;

    let order_id_bytes = ctx.accounts.order.order_id.to_le_bytes();
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
        refund_amount,
        ctx.accounts.token_mint.decimals,
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
