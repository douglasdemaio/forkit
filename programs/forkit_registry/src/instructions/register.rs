use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
#[instruction(role: Role)]
pub struct Register<'info> {
    #[account(
        init,
        payer = wallet,
        space = Profile::SPACE,
        seeds = [Profile::SEED, wallet.key().as_ref(), &[role as u8]],
        bump,
    )]
    pub profile: Account<'info, Profile>,
    #[account(mut)]
    pub wallet: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Register>, role: Role, metadata_uri: String) -> Result<()> {
    require!(metadata_uri.len() <= MAX_METADATA_URI_LEN, ErrorCode::ConstraintRaw);

    let clock = Clock::get()?;
    let profile = &mut ctx.accounts.profile;

    profile.wallet = ctx.accounts.wallet.key();
    profile.role = role;
    profile.trust_score = 5000; // Start at 50.00
    profile.completed_orders = 0;
    profile.total_ratings = 0;
    profile.sum_ratings = 0;
    profile.disputes_filed = 0;
    profile.disputes_lost = 0;
    profile.loyalty_points = 0;
    profile.registered_at = clock.unix_timestamp;
    profile.last_active_at = clock.unix_timestamp;
    profile.metadata_uri = metadata_uri;
    profile.is_active = true;
    profile.bump = ctx.bumps.profile;

    emit!(ProfileRegistered {
        wallet: profile.wallet,
        role,
    });

    Ok(())
}
