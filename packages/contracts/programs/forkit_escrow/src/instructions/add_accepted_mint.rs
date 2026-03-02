use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::ForkitError;

#[derive(Accounts)]
pub struct AddAcceptedMint<'info> {
    #[account(
        mut,
        seeds = [ProtocolConfig::SEED],
        bump = protocol_config.bump,
        has_one = admin @ ForkitError::Unauthorized,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    pub mint: Account<'info, anchor_spl::token::Mint>,
    pub admin: Signer<'info>,
}

pub fn handler(ctx: Context<AddAcceptedMint>) -> Result<()> {
    let config = &mut ctx.accounts.protocol_config;
    let mint_key = ctx.accounts.mint.key();

    require!(
        config.accepted_mints.len() < MAX_ACCEPTED_MINTS,
        ForkitError::MaxMintsReached
    );
    require!(
        !config.accepted_mints.contains(&mint_key),
        ForkitError::MintAlreadyAccepted
    );

    config.accepted_mints.push(mint_key);
    Ok(())
}
