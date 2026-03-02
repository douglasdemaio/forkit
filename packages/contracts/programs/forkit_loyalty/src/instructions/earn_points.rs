use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct EarnPoints<'info> {
    #[account(
        init_if_needed,
        payer = wallet,
        space = LoyaltyAccount::SPACE,
        seeds = [LoyaltyAccount::SEED, wallet.key().as_ref()],
        bump,
    )]
    pub loyalty_account: Account<'info, LoyaltyAccount>,
    #[account(mut)]
    pub wallet: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<EarnPoints>, amount: u64) -> Result<()> {
    let account = &mut ctx.accounts.loyalty_account;
    account.wallet = ctx.accounts.wallet.key();
    account.points = account.points.saturating_add(amount);
    account.lifetime_earned = account.lifetime_earned.saturating_add(amount);
    account.bump = ctx.bumps.loyalty_account;
    Ok(())
}
