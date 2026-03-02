use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct RedeemPoints<'info> {
    #[account(
        mut,
        seeds = [LoyaltyAccount::SEED, wallet.key().as_ref()],
        bump = loyalty_account.bump,
        has_one = wallet,
    )]
    pub loyalty_account: Account<'info, LoyaltyAccount>,
    pub wallet: Signer<'info>,
}

pub fn handler(ctx: Context<RedeemPoints>, amount: u64) -> Result<()> {
    let account = &mut ctx.accounts.loyalty_account;
    require!(account.points >= amount, ErrorCode::InsufficientFunds);
    account.points = account.points.saturating_sub(amount);
    account.lifetime_redeemed = account.lifetime_redeemed.saturating_add(amount);
    Ok(())
}
