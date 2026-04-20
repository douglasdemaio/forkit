use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::constants::*;
use crate::errors::TokenError;

// ── request_withdrawal ────────────────────────────────────────────────────────

/// Create a withdrawal request. Any of the 5 multisig signers may propose one.
/// The request is executed once 3 distinct signers have approved.
#[derive(Accounts)]
pub struct RequestWithdrawal<'info> {
    #[account(
        mut,
        seeds = [RESERVE_CONFIG_SEED],
        bump = reserve_config.bump,
    )]
    pub reserve_config: Account<'info, ReserveConfig>,

    #[account(
        init,
        payer = proposer,
        space = WithdrawalRequest::SPACE,
        seeds = [WITHDRAWAL_SEED, &reserve_config.withdrawal_nonce.to_le_bytes()],
        bump,
    )]
    pub withdrawal_request: Account<'info, WithdrawalRequest>,

    #[account(mut)]
    pub proposer: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn request_withdrawal_handler(
    ctx: Context<RequestWithdrawal>,
    amount: u64,
    token_mint: Pubkey,
    recipient: Pubkey,
) -> Result<()> {
    // Validate proposer is a registered signer
    require!(
        ctx.accounts.reserve_config.multisig_signers
            .iter()
            .any(|s| *s == ctx.accounts.proposer.key()),
        TokenError::NotMultisigSigner
    );

    let clock = Clock::get()?;
    let nonce = ctx.accounts.reserve_config.withdrawal_nonce;

    let req = &mut ctx.accounts.withdrawal_request;
    req.nonce = nonce;
    req.recipient = recipient;
    req.amount = amount;
    req.token_mint = token_mint;
    req.approval_bitmask = 0;
    req.executed = false;
    req.created_at = clock.unix_timestamp;
    req.bump = ctx.bumps.withdrawal_request;

    ctx.accounts.reserve_config.withdrawal_nonce = nonce
        .checked_add(1)
        .ok_or(TokenError::ArithmeticOverflow)?;

    emit!(WithdrawalRequested { nonce, recipient, amount, token_mint });

    Ok(())
}

// ── approve_withdrawal ────────────────────────────────────────────────────────

/// Each multisig signer calls this to approve a pending withdrawal.
/// The request executes automatically once MULTISIG_THRESHOLD (3) approvals
/// have been collected (via execute_withdrawal).
#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct ApproveWithdrawal<'info> {
    #[account(
        seeds = [RESERVE_CONFIG_SEED],
        bump = reserve_config.bump,
    )]
    pub reserve_config: Account<'info, ReserveConfig>,

    #[account(
        mut,
        seeds = [WITHDRAWAL_SEED, &nonce.to_le_bytes()],
        bump = withdrawal_request.bump,
        constraint = !withdrawal_request.executed @ TokenError::WithdrawalAlreadyExecuted,
    )]
    pub withdrawal_request: Account<'info, WithdrawalRequest>,

    pub signer: Signer<'info>,
}

pub fn approve_withdrawal_handler(
    ctx: Context<ApproveWithdrawal>,
    nonce: u64,
) -> Result<()> {
    let signers = &ctx.accounts.reserve_config.multisig_signers;
    let signer_key = ctx.accounts.signer.key();

    // Find this signer's index in the multisig list
    let signer_idx = signers
        .iter()
        .position(|s| *s == signer_key)
        .ok_or(TokenError::NotMultisigSigner)? as u8;

    let req = &mut ctx.accounts.withdrawal_request;
    let bit = 1u8 << signer_idx;

    require!(req.approval_bitmask & bit == 0, TokenError::AlreadyApproved);
    req.approval_bitmask |= bit;

    let approvals_count = req.approvals_count();
    emit!(WithdrawalApproved {
        nonce,
        signer: signer_key,
        approvals_count,
    });

    Ok(())
}

// ── execute_withdrawal ────────────────────────────────────────────────────────

/// Permissionless — execute a withdrawal request once 3 approvals are on record.
#[derive(Accounts)]
#[instruction(nonce: u64)]
pub struct ExecuteWithdrawal<'info> {
    #[account(
        seeds = [TOKEN_CONFIG_SEED],
        bump = token_config.bump,
    )]
    pub token_config: Account<'info, TokenConfig>,

    #[account(
        seeds = [RESERVE_CONFIG_SEED],
        bump = reserve_config.bump,
    )]
    pub reserve_config: Account<'info, ReserveConfig>,

    #[account(
        mut,
        seeds = [WITHDRAWAL_SEED, &nonce.to_le_bytes()],
        bump = withdrawal_request.bump,
        constraint = !withdrawal_request.executed @ TokenError::WithdrawalAlreadyExecuted,
        constraint = withdrawal_request.approvals_count() >= MULTISIG_THRESHOLD
            @ TokenError::InsufficientApprovals,
    )]
    pub withdrawal_request: Account<'info, WithdrawalRequest>,

    /// Reserve vault to withdraw from — must match the request's token_mint.
    #[account(
        mut,
        seeds = [RESERVE_VAULT_SEED, withdrawal_request.token_mint.as_ref()],
        bump,
        constraint = reserve_vault.mint == withdrawal_request.token_mint
            @ TokenError::UnsupportedMint,
    )]
    pub reserve_vault: Account<'info, TokenAccount>,

    /// Recipient token account — must match request's recipient and token_mint.
    #[account(
        mut,
        constraint = recipient_token_account.owner == withdrawal_request.recipient
            @ TokenError::Unauthorized,
        constraint = recipient_token_account.mint == withdrawal_request.token_mint
            @ TokenError::UnsupportedMint,
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    pub caller: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn execute_withdrawal_handler(
    ctx: Context<ExecuteWithdrawal>,
    nonce: u64,
) -> Result<()> {
    let req = &mut ctx.accounts.withdrawal_request;
    req.executed = true;

    let amount = req.amount;
    let recipient = req.recipient;
    let token_mint = req.token_mint;

    let config_seeds: &[&[u8]] = &[TOKEN_CONFIG_SEED, &[ctx.accounts.token_config.bump]];
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.reserve_vault.to_account_info(),
                to: ctx.accounts.recipient_token_account.to_account_info(),
                authority: ctx.accounts.token_config.to_account_info(),
            },
            &[config_seeds],
        ),
        amount,
    )?;

    emit!(WithdrawalExecuted { nonce, recipient, amount, token_mint });

    Ok(())
}
