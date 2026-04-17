use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct UpdateMetadata<'info> {
    #[account(
        mut,
        has_one = wallet,
    )]
    pub profile: Account<'info, Profile>,
    pub wallet: Signer<'info>,
}

pub fn handler(ctx: Context<UpdateMetadata>, new_uri: String) -> Result<()> {
    require!(new_uri.len() <= MAX_METADATA_URI_LEN, ErrorCode::ConstraintRaw);
    ctx.accounts.profile.metadata_uri = new_uri;
    Ok(())
}
