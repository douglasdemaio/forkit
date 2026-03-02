use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
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
    pub token_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [b"escrow_vault", &order.order_id.to_le_bytes()],
        bump,
    )]
    pub escrow_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = contributor_token_account.owner == contributor.key(),
        constraint = contributor_token_account.mint == token_mint.key(),
    )]
    pub contributor_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub contributor: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<ContributeToOrder>, amount: u64) -> Result<()> {
    let order = &ctx.accounts.order;

    require!(amount > 0, ForkitError::ZeroContribution);
    require!(
        order.status == OrderStatus::Created,
        ForkitError::InvalidOrderStatus
    );
    require!(!order.is_fully_funded(), ForkitError::AlreadyFullyFunded);

    // Check funding timeout
    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp <= order.created_at + FUNDING_TIMEOUT_SECONDS,
        ForkitError::FundingExpired
    );

    // Cap contribution at remaining needed
    let remaining = order.escrow_target
        .checked_sub(order.escrow_funded)
        .ok_or(ForkitError::ArithmeticOverflow)?;
    let actual_amount = amount.min(remaining);

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
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.contributor_token_account.to_account_info(),
                to: ctx.accounts.escrow_vault.to_account_info(),
                authority: ctx.accounts.contributor.to_account_info(),
            },
        ),
        actual_amount,
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
