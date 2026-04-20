use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = TokenConfig::SPACE,
        seeds = [TOKEN_CONFIG_SEED],
        bump,
    )]
    pub token_config: Account<'info, TokenConfig>,

    /// FORKIT SPL mint — PDA-owned so only this program can mint.
    /// Anchor 0.30 omitting mint::freeze_authority initialises with None —
    /// tokens are freely transferable. If your anchor version defaults to
    /// authority instead, add a post-init set_authority CPI to clear it.
    #[account(
        init,
        payer = admin,
        seeds = [FORKIT_MINT_SEED],
        bump,
        mint::decimals = FORKIT_DECIMALS,
        mint::authority = token_config,
    )]
    pub forkit_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        space = ReserveConfig::SPACE,
        seeds = [RESERVE_CONFIG_SEED],
        bump,
    )]
    pub reserve_config: Account<'info, ReserveConfig>,

    /// USDC reserve vault — program-owned PDA holds reserve USDC
    #[account(
        init,
        payer = admin,
        seeds = [RESERVE_VAULT_SEED, usdc_mint.key().as_ref()],
        bump,
        token::mint = usdc_mint,
        token::authority = token_config,
    )]
    pub reserve_usdc_vault: Account<'info, TokenAccount>,

    pub usdc_mint: Account<'info, Mint>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<Initialize>,
    multisig_signers: [Pubkey; 5],
) -> Result<()> {
    let token_config = &mut ctx.accounts.token_config;
    token_config.mint = ctx.accounts.forkit_mint.key();
    token_config.owner = ctx.accounts.admin.key();
    token_config.mint_rate = DEFAULT_MINT_RATE;
    token_config.pending_mint_rate = 0;
    token_config.rate_change_proposed_at = 0;
    token_config.total_supply = 0;
    token_config.global_obligation_count = 0;
    token_config.last_batch_at = Clock::get()?.unix_timestamp;
    token_config.locked = false;
    token_config.bump = ctx.bumps.token_config;
    token_config.mint_bump = ctx.bumps.forkit_mint;

    let reserve_config = &mut ctx.accounts.reserve_config;
    reserve_config.usdc_bps = DEFAULT_RESERVE_USDC_BPS;
    reserve_config.sol_bps = DEFAULT_RESERVE_SOL_BPS;
    reserve_config.btc_bps = DEFAULT_RESERVE_BTC_BPS;
    reserve_config.pending_usdc_bps = 0;
    reserve_config.pending_sol_bps = 0;
    reserve_config.pending_btc_bps = 0;
    reserve_config.pending_effective_at = 0;
    reserve_config.multisig_signers = multisig_signers;
    reserve_config.withdrawal_nonce = 0;
    reserve_config.locked = false;
    reserve_config.bump = ctx.bumps.reserve_config;

    emit!(ForkitTokenInitialized {
        mint: ctx.accounts.forkit_mint.key(),
        owner: ctx.accounts.admin.key(),
    });

    Ok(())
}
