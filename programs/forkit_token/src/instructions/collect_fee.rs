use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::constants::*;
use crate::errors::TokenError;

/// Collect the 0.02% protocol fee from a customer payment and route it equally
/// to four destinations:
///
///   0.005% → platform wallet (USDC transfer)
///   0.005% → restaurant wallet (USDC transfer)
///   0.005% → reserve vault (USDC, later converted to basket)
///   0.005% → reserve vault as FORKIT-backing (obligation recorded for customer)
///
/// The four splits are validated to sum to exactly the computed total fee before
/// any transfers execute — the transaction is rejected if they don't.
///
/// Callable standalone or via CPI from forkit_escrow's confirm_delivery.
/// The source_authority must be a signer (regular wallet or escrow PDA via CPI).
#[derive(Accounts)]
pub struct CollectFee<'info> {
    #[account(
        mut,
        seeds = [TOKEN_CONFIG_SEED],
        bump = token_config.bump,
        constraint = !token_config.locked @ TokenError::Reentrancy,
    )]
    pub token_config: Account<'info, TokenConfig>,

    /// Accumulates the customer's pending FORKIT mint obligation.
    #[account(
        init_if_needed,
        payer = source_authority,
        space = MintObligation::SPACE,
        seeds = [MINT_OBLIGATION_SEED, customer.key().as_ref()],
        bump,
    )]
    pub mint_obligation: Account<'info, MintObligation>,

    /// Source of the USDC fee (escrow vault or customer ATA).
    #[account(
        mut,
        constraint = source_account.mint == usdc_mint.key() @ TokenError::UnsupportedMint,
    )]
    pub source_account: Account<'info, TokenAccount>,

    /// Authority over source_account — signer (regular or PDA via CPI).
    #[account(mut)]
    pub source_authority: Signer<'info>,

    pub usdc_mint: Account<'info, Mint>,

    /// Platform USDC ATA — must be owned by PLATFORM_WALLET.
    #[account(
        mut,
        constraint = platform_token_account.owner == PLATFORM_WALLET
            @ TokenError::InvalidPlatformWallet,
        constraint = platform_token_account.mint == usdc_mint.key()
            @ TokenError::UnsupportedMint,
    )]
    pub platform_token_account: Account<'info, TokenAccount>,

    /// Restaurant USDC ATA.
    #[account(
        mut,
        constraint = restaurant_token_account.mint == usdc_mint.key()
            @ TokenError::UnsupportedMint,
    )]
    pub restaurant_token_account: Account<'info, TokenAccount>,

    /// Reserve USDC vault (PDA-owned) — receives both reserve_fee + customer_fee
    /// backing so that every minted FORKIT has USDC collateral.
    #[account(
        mut,
        seeds = [RESERVE_VAULT_SEED, usdc_mint.key().as_ref()],
        bump,
        constraint = reserve_usdc_vault.mint == usdc_mint.key() @ TokenError::UnsupportedMint,
    )]
    pub reserve_usdc_vault: Account<'info, TokenAccount>,

    /// Customer wallet — receives the FORKIT mint obligation (not USDC).
    /// CHECK: used only as a key for the obligation PDA seed.
    pub customer: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<CollectFee>, amount: u64) -> Result<()> {
    // ── Reentrancy lock ──────────────────────────────────────────────────────
    ctx.accounts.token_config.locked = true;

    // ── Compute splits ───────────────────────────────────────────────────────
    let total_fee = (amount as u128)
        .checked_mul(TOTAL_FEE_PPM as u128)
        .ok_or(TokenError::ArithmeticOverflow)?
        .checked_div(1_000_000)
        .ok_or(TokenError::ArithmeticOverflow)? as u64;

    // Edge case: payment too small for a non-zero fee → no-op (valid per spec)
    if total_fee == 0 {
        ctx.accounts.token_config.locked = false;
        return Ok(());
    }

    let quarter = total_fee / 4;
    let platform_fee = quarter;
    let customer_fee = quarter;   // USDC backing for FORKIT mint
    let restaurant_fee = quarter;
    // Reserve absorbs any rounding remainder so sum always equals total_fee
    let reserve_fee = total_fee
        .checked_sub(platform_fee)
        .and_then(|r| r.checked_sub(customer_fee))
        .and_then(|r| r.checked_sub(restaurant_fee))
        .ok_or(TokenError::ArithmeticOverflow)?;

    // ── Mandatory sum verification ────────────────────────────────────────────
    let sum = platform_fee
        .checked_add(customer_fee)
        .and_then(|s| s.checked_add(restaurant_fee))
        .and_then(|s| s.checked_add(reserve_fee))
        .ok_or(TokenError::ArithmeticOverflow)?;
    require!(sum == total_fee, TokenError::FeeSplitMismatch);

    // ── Transfer platform fee ────────────────────────────────────────────────
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.source_account.to_account_info(),
                to: ctx.accounts.platform_token_account.to_account_info(),
                authority: ctx.accounts.source_authority.to_account_info(),
            },
        ),
        platform_fee,
    )?;

    // ── Transfer restaurant fee ──────────────────────────────────────────────
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.source_account.to_account_info(),
                to: ctx.accounts.restaurant_token_account.to_account_info(),
                authority: ctx.accounts.source_authority.to_account_info(),
            },
        ),
        restaurant_fee,
    )?;

    // ── Transfer customer_fee + reserve_fee → reserve vault ──────────────────
    // customer_fee USDC becomes collateral backing for the FORKIT obligation;
    // reserve_fee USDC is for basket conversion — both are program-held.
    let reserve_deposit = customer_fee
        .checked_add(reserve_fee)
        .ok_or(TokenError::ArithmeticOverflow)?;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.source_account.to_account_info(),
                to: ctx.accounts.reserve_usdc_vault.to_account_info(),
                authority: ctx.accounts.source_authority.to_account_info(),
            },
        ),
        reserve_deposit,
    )?;

    // ── Compute FORKIT obligation for customer ───────────────────────────────
    // mint_rate: FORKIT units per 1 USDC unit of fee
    let forkit_obligation = (customer_fee as u128)
        .checked_mul(ctx.accounts.token_config.mint_rate as u128)
        .ok_or(TokenError::ArithmeticOverflow)? as u64;

    // ── Record obligation ─────────────────────────────────────────────────────
    let obligation = &mut ctx.accounts.mint_obligation;
    if obligation.wallet == Pubkey::default() {
        obligation.wallet = ctx.accounts.customer.key();
        obligation.bump = ctx.bumps.mint_obligation;
    }
    obligation.pending_forkit = obligation.pending_forkit
        .checked_add(forkit_obligation)
        .ok_or(TokenError::ArithmeticOverflow)?;

    // ── Update global batch counter ───────────────────────────────────────────
    ctx.accounts.token_config.global_obligation_count = ctx
        .accounts.token_config.global_obligation_count
        .checked_add(1)
        .ok_or(TokenError::ArithmeticOverflow)?;

    emit!(FeesCollected {
        customer: ctx.accounts.customer.key(),
        payment_amount: amount,
        platform_fee,
        restaurant_fee,
        reserve_fee,
        customer_fee_usdc: customer_fee,
        forkit_obligation,
    });

    // ── Release reentrancy lock ──────────────────────────────────────────────
    ctx.accounts.token_config.locked = false;
    Ok(())
}
