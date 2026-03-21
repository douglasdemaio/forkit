use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::LoyaltyError;

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

/// Burn `amount` points from the caller's loyalty balance (e.g., to apply an order discount).
pub fn handler(ctx: Context<RedeemPoints>, amount: u64) -> Result<()> {
    require!(amount > 0, LoyaltyError::ZeroPoints);
    let account = &mut ctx.accounts.loyalty_account;
    require!(account.points >= amount, LoyaltyError::InsufficientPoints);

    account.points = account.points.saturating_sub(amount);
    account.lifetime_redeemed = account.lifetime_redeemed.saturating_add(amount);

    emit!(PointsRedeemed {
        wallet: ctx.accounts.wallet.key(),
        amount,
        new_balance: account.points,
    });

    Ok(())
}
