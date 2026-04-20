use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount};
use crate::state::*;
use crate::constants::*;
use crate::errors::TokenError;

/// Execute a pending FORKIT mint for one customer.
///
/// Callable by anyone once the batch threshold is reached:
///   • At least MINT_BATCH_TX_THRESHOLD (100) fee obligations have accumulated, OR
///   • At least MINT_BATCH_TIME_THRESHOLD (1 hour) has elapsed since the last batch.
///
/// Each call mints for exactly one customer to keep compute units bounded.
/// The caller passes the customer's wallet; the obligation PDA is derived from it.
/// Creates the customer's FORKIT ATA if it does not yet exist.
#[derive(Accounts)]
pub struct ExecuteMintBatch<'info> {
    #[account(
        mut,
        seeds = [TOKEN_CONFIG_SEED],
        bump = token_config.bump,
        constraint = !token_config.locked @ TokenError::Reentrancy,
    )]
    pub token_config: Account<'info, TokenConfig>,

    #[account(
        mut,
        seeds = [FORKIT_MINT_SEED],
        bump = token_config.mint_bump,
    )]
    pub forkit_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [MINT_OBLIGATION_SEED, customer.key().as_ref()],
        bump = mint_obligation.bump,
        constraint = mint_obligation.pending_forkit > 0 @ TokenError::NoPendingObligation,
    )]
    pub mint_obligation: Account<'info, MintObligation>,

    /// Customer's FORKIT ATA — created here if it doesn't exist.
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = forkit_mint,
        associated_token::authority = customer,
    )]
    pub customer_forkit_ata: Account<'info, TokenAccount>,

    /// CHECK: only used as a key to derive the obligation PDA and ATA
    pub customer: UncheckedAccount<'info>,

    /// Permissionless — anyone can trigger a batch execution.
    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<ExecuteMintBatch>) -> Result<()> {
    let clock = Clock::get()?;
    let config = &ctx.accounts.token_config;

    // ── Batch readiness check ─────────────────────────────────────────────────
    let tx_threshold_met = config.global_obligation_count >= MINT_BATCH_TX_THRESHOLD;
    let time_threshold_met =
        clock.unix_timestamp - config.last_batch_at >= MINT_BATCH_TIME_THRESHOLD;
    require!(tx_threshold_met || time_threshold_met, TokenError::BatchNotReady);

    // ── Reentrancy lock ──────────────────────────────────────────────────────
    ctx.accounts.token_config.locked = true;

    let amount = ctx.accounts.mint_obligation.pending_forkit;
    let obligation_count = ctx.accounts.token_config.global_obligation_count;

    // ── Mint FORKIT directly to customer ATA ──────────────────────────────────
    let config_seeds: &[&[u8]] = &[TOKEN_CONFIG_SEED, &[ctx.accounts.token_config.bump]];
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.forkit_mint.to_account_info(),
                to: ctx.accounts.customer_forkit_ata.to_account_info(),
                authority: ctx.accounts.token_config.to_account_info(),
            },
            &[config_seeds],
        ),
        amount,
    )?;

    // ── Update state ─────────────────────────────────────────────────────────
    ctx.accounts.mint_obligation.pending_forkit = 0;

    ctx.accounts.token_config.total_supply = ctx
        .accounts.token_config.total_supply
        .checked_add(amount)
        .ok_or(TokenError::ArithmeticOverflow)?;

    // Decrement global counter — saturate at zero to handle concurrent calls
    ctx.accounts.token_config.global_obligation_count =
        ctx.accounts.token_config.global_obligation_count.saturating_sub(1);

    // Reset batch timer when counter reaches zero (full batch complete)
    if ctx.accounts.token_config.global_obligation_count == 0 {
        ctx.accounts.token_config.last_batch_at = clock.unix_timestamp;
    }

    emit!(TokensMinted {
        customer: ctx.accounts.customer.key(),
        amount_minted: amount,
        new_total_supply: ctx.accounts.token_config.total_supply,
        obligation_count_at_trigger: obligation_count,
    });

    ctx.accounts.token_config.locked = false;
    Ok(())
}
