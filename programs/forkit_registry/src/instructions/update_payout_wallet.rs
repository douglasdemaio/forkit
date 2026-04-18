use anchor_lang::prelude::*;
use crate::state::*;

#[derive(Accounts)]
pub struct UpdatePayoutWallet<'info> {
    #[account(
        mut,
        seeds = [Profile::SEED, wallet.key().as_ref(), &[profile.role as u8]],
        bump = profile.bump,
        has_one = wallet,
    )]
    pub profile: Account<'info, Profile>,

    /// Only the profile owner (operational wallet) can change the payout wallet.
    /// This ensures employees with signing access control where funds go,
    /// and every change is recorded on-chain for auditing.
    pub wallet: Signer<'info>,
}

pub fn handler(ctx: Context<UpdatePayoutWallet>, new_payout_wallet: Pubkey) -> Result<()> {
    let profile = &mut ctx.accounts.profile;
    let clock = Clock::get()?;

    let old_payout_wallet = profile.payout_wallet;

    // Don't emit event if nothing changed
    require!(
        old_payout_wallet != new_payout_wallet,
        ErrorCode::ConstraintRaw
    );

    profile.payout_wallet = new_payout_wallet;
    profile.last_active_at = clock.unix_timestamp;

    emit!(PayoutWalletChanged {
        wallet: profile.wallet,
        old_payout_wallet,
        new_payout_wallet,
        changed_at: clock.unix_timestamp,
    });

    Ok(())
}
