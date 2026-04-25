use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, TokenInterface, TokenAccount, TransferChecked, Mint};
use crate::state::*;
use crate::errors::ForkitError;

#[derive(Accounts)]
pub struct ContributeToOrder<'info> {
    #[account(
        mut,
        seeds = [Order::SEED, &order.order_id.to_le_bytes()],
        bump = order.bump,
    )]
    pub order: Account<'info, Order>,

    /// Contribution PDA — init_if_needed for new contributors, existing for repeat
    #[account(
        init_if_needed,
        payer = contributor,
        space = Contribution::SPACE,
        seeds = [Contribution::SEED, &order.order_id.to_le_bytes(), contributor.key().as_ref()],
        bump,
    )]
    pub contribution: Account<'info, Contribution>,

    #[account(
        constraint = token_mint.key() == order.token_mint @ ForkitError::UnsupportedMint,
    )]
    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        mut,
        seeds = [ESCROW_VAULT_SEED, &order.order_id.to_le_bytes()],
        bump,
    )]
    pub escrow_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = contributor_token_account.owner == contributor.key(),
        constraint = contributor_token_account.mint == token_mint.key(),
    )]
    pub contributor_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub contributor: Signer<'info>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<ContributeToOrder>, amount: u64) -> Result<()> {
    let order = &ctx.accounts.order;

    require!(amount > 0, ForkitError::ZeroContribution);
    // Allow contributions while Created (pre-funding) or Funded (post-funding
    // reimbursement model: friends chip in to reimburse the original payer).
    require!(
        order.status == OrderStatus::Created || order.status == OrderStatus::Funded
            || order.status == OrderStatus::Preparing || order.status == OrderStatus::ReadyForPickup,
        ForkitError::InvalidOrderStatus
    );

    // Funding timeout only applies to initial funding (Created status).
    // Reimbursement contributions (Funded/Preparing/ReadyForPickup) are unconstrained
    // by time — contributors reimburse the original payer after the fact.
    if order.status == OrderStatus::Created {
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp <= order.created_at + FUNDING_TIMEOUT_SECONDS,
            ForkitError::FundingExpired
        );
    }

    // If not yet fully funded, cap at remaining. If already funded, accept
    // the full amount — it will be available as reimbursement to earlier contributors.
    let actual_amount = if order.escrow_funded < order.escrow_target {
        let remaining = order.escrow_target
            .checked_sub(order.escrow_funded)
            .ok_or(ForkitError::ArithmeticOverflow)?;
        amount.min(remaining)
    } else {
        // Already funded — this is a reimbursement contribution.
        // Cap at escrow_target so nobody can overfund beyond 2× the order.
        let max_overfund = order.escrow_target;
        let current_excess = order.escrow_funded.saturating_sub(order.escrow_target);
        let room = max_overfund.saturating_sub(current_excess);
        amount.min(room)
    };

    require!(actual_amount > 0, ForkitError::AlreadyFullyFunded);

    // Track if this is a new contributor
    let contribution = &mut ctx.accounts.contribution;
    let is_new = contribution.amount == 0 && contribution.contributor == Pubkey::default();

    if is_new {
        require!(
            (order.contributor_count as usize) < MAX_CONTRIBUTORS,
            ForkitError::MaxContributorsReached
        );
        contribution.order_id = order.order_id;
        contribution.contributor = ctx.accounts.contributor.key();
        contribution.bump = ctx.bumps.contribution;
    }

    // Transfer tokens to escrow
    token_interface::transfer_checked(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            TransferChecked {
                from: ctx.accounts.contributor_token_account.to_account_info(),
                mint: ctx.accounts.token_mint.to_account_info(),
                to: ctx.accounts.escrow_vault.to_account_info(),
                authority: ctx.accounts.contributor.to_account_info(),
            },
        ),
        actual_amount,
        ctx.accounts.token_mint.decimals,
    )?;

    contribution.amount = contribution.amount
        .checked_add(actual_amount)
        .ok_or(ForkitError::ArithmeticOverflow)?;

    let order = &mut ctx.accounts.order;
    order.escrow_funded = order.escrow_funded
        .checked_add(actual_amount)
        .ok_or(ForkitError::ArithmeticOverflow)?;

    if is_new {
        order.contributor_count = order.contributor_count
            .checked_add(1)
            .ok_or(ForkitError::ArithmeticOverflow)?;
    }

    let fully_funded = order.is_fully_funded();
    if fully_funded {
        order.status = OrderStatus::Funded;
        emit!(OrderFunded {
            order_id: order.order_id,
            total_funded: order.escrow_funded,
            contributor_count: order.contributor_count,
        });
    }

    emit!(ContributionMade {
        order_id: order.order_id,
        contributor: ctx.accounts.contributor.key(),
        amount: actual_amount,
        total_funded: order.escrow_funded,
        escrow_target: order.escrow_target,
        fully_funded,
    });

    Ok(())
}
