use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::LoyaltyError;

/// AI-routed orders earn a 50% bonus on top of the base points.
const AI_ORDER_BONUS_BPS: u64 = 5000;

#[derive(Accounts)]
pub struct EarnPoints<'info> {
    #[account(
        seeds = [LoyaltyConfig::SEED],
        bump = loyalty_config.bump,
    )]
    pub loyalty_config: Account<'info, LoyaltyConfig>,

    #[account(
        init_if_needed,
        payer = authority,
        space = LoyaltyAccount::SPACE,
        seeds = [LoyaltyAccount::SEED, recipient.key().as_ref()],
        bump,
    )]
    pub loyalty_account: Account<'info, LoyaltyAccount>,

    /// CHECK: Recipient wallet whose loyalty account receives points.
    pub recipient: UncheckedAccount<'info>,

    /// Must match loyalty_config.authorized_escrow — only the escrow program
    /// (or its designated authority) may award loyalty points.
    #[account(
        mut,
        constraint = authority.key() == loyalty_config.authorized_escrow
            @ LoyaltyError::UnauthorizedCaller,
    )]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

/// Award `base_amount` loyalty points to `recipient`.
/// If `is_ai_order` is true, a 50% bonus is applied (rewarding AI-routed deliveries).
/// Only `loyalty_config.authorized_escrow` may call this instruction.
pub fn handler(ctx: Context<EarnPoints>, base_amount: u64, is_ai_order: bool) -> Result<()> {
    require!(base_amount > 0, LoyaltyError::ZeroPoints);

    let account = &mut ctx.accounts.loyalty_account;
    let config = &mut ctx.accounts.loyalty_config;
    let clock = Clock::get()?;

    // Apply AI routing bonus
    let bonus = if is_ai_order {
        base_amount
            .checked_mul(AI_ORDER_BONUS_BPS)
            .ok_or(LoyaltyError::ArithmeticOverflow)?
            .checked_div(10_000)
            .ok_or(LoyaltyError::ArithmeticOverflow)?
    } else {
        0
    };
    let total_award = base_amount
        .checked_add(bonus)
        .ok_or(LoyaltyError::ArithmeticOverflow)?;

    // Initialize wallet if first time
    if account.wallet == Pubkey::default() {
        account.wallet = ctx.accounts.recipient.key();
        account.bump = ctx.bumps.loyalty_account;
    }

    account.points = account.points.saturating_add(total_award);
    account.lifetime_earned = account.lifetime_earned.saturating_add(total_award);
    account.last_earned_at = clock.unix_timestamp;

    if is_ai_order {
        account.ai_orders_count = account.ai_orders_count.saturating_add(1);
    }

    // Update protocol total
    config.total_points_issued = config.total_points_issued.saturating_add(total_award);

    // Check for tier upgrade
    let tier_upgraded = account.update_tier();

    emit!(PointsEarned {
        wallet: ctx.accounts.recipient.key(),
        amount: total_award,
        is_ai_order,
        new_balance: account.points,
        lifetime_earned: account.lifetime_earned,
    });

    if tier_upgraded {
        emit!(TierUpgraded {
            wallet: ctx.accounts.recipient.key(),
            new_tier: account.tier,
            lifetime_earned: account.lifetime_earned,
        });
    }

    Ok(())
}
