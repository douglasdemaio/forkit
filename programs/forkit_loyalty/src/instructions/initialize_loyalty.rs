use anchor_lang::prelude::*;
use crate::state::LoyaltyConfig;

#[derive(Accounts)]
pub struct InitializeLoyalty<'info> {
    #[account(
        init,
        payer = admin,
        space = LoyaltyConfig::SPACE,
        seeds = [LoyaltyConfig::SEED],
        bump,
    )]
    pub loyalty_config: Account<'info, LoyaltyConfig>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Initialize the loyalty program config, binding it to the authorized escrow caller.
/// `authorized_escrow` should be the escrow program's PDA authority or admin wallet
/// that will sign earn_points CPIs.
pub fn handler(ctx: Context<InitializeLoyalty>, authorized_escrow: Pubkey) -> Result<()> {
    let config = &mut ctx.accounts.loyalty_config;
    config.admin = ctx.accounts.admin.key();
    config.authorized_escrow = authorized_escrow;
    config.total_points_issued = 0;
    config.bump = ctx.bumps.loyalty_config;
    Ok(())
}
