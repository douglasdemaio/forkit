use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::ForkitError;

#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    #[account(
        init,
        payer = admin,
        space = ProtocolConfig::space(MAX_ACCEPTED_MINTS),
        seeds = [ProtocolConfig::SEED],
        bump,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeProtocol>, fee_basis_points: u16) -> Result<()> {
    require!(fee_basis_points <= 10000, ForkitError::InvalidFeeBasisPoints);

    let config = &mut ctx.accounts.protocol_config;
    config.admin = ctx.accounts.admin.key();
    // Treasury: BiP5PJuUiXPYCFx98RMCGCnRhdUVrkxSke9C6y2ZohQ9
    config.treasury_wallet = Pubkey::default(); // Set via update_protocol_config after init
    config.fee_basis_points = fee_basis_points;
    config.accepted_mints = Vec::new();
    config.bump = ctx.bumps.protocol_config;

    Ok(())
}
