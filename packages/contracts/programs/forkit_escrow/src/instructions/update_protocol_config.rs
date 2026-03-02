use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::ForkitError;

#[derive(Accounts)]
pub struct UpdateProtocolConfig<'info> {
    #[account(
        mut,
        seeds = [ProtocolConfig::SEED],
        bump = protocol_config.bump,
        has_one = admin @ ForkitError::Unauthorized,
    )]
    pub protocol_config: Account<'info, ProtocolConfig>,
    pub admin: Signer<'info>,
}

pub fn handler(
    ctx: Context<UpdateProtocolConfig>,
    new_fee_basis_points: Option<u16>,
    new_treasury: Option<Pubkey>,
) -> Result<()> {
    let config = &mut ctx.accounts.protocol_config;

    if let Some(fee) = new_fee_basis_points {
        require!(fee <= 10000, ForkitError::InvalidFeeBasisPoints);
        config.fee_basis_points = fee;
    }

    if let Some(treasury) = new_treasury {
        config.treasury_wallet = treasury;
    }

    Ok(())
}
