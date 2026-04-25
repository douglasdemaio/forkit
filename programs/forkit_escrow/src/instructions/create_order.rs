use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, TokenInterface, TokenAccount, TransferChecked, Mint};
use crate::state::*;
use crate::errors::ForkitError;

/// Loyalty points awarded = 1% of (food_amount + delivery_amount) in base units.
/// The loyalty program scales this further (e.g. AI bonus).
pub const LOYALTY_POINTS_BPS: u64 = 100;

#[derive(Accounts)]
#[instruction(order_id: u64)]
pub struct CreateOrder<'info> {
    #[account(
        init,
        payer = customer,
        space = Order::SPACE,
        seeds = [Order::SEED, &order_id.to_le_bytes()],
        bump,
    )]
    pub order: Box<Account<'info, Order>>,

    /// The creator's contribution record (may be 0 if they don't fund yet)
    #[account(
        init,
        payer = customer,
        space = Contribution::SPACE,
        seeds = [Contribution::SEED, &order_id.to_le_bytes(), customer.key().as_ref()],
        bump,
    )]
    pub contribution: Box<Account<'info, Contribution>>,

    #[account(
        seeds = [ProtocolConfig::SEED],
        bump = protocol_config.bump,
    )]
    pub protocol_config: Box<Account<'info, ProtocolConfig>>,

    /// CHECK: Validated by the caller; restaurant identity stored on the order
    pub restaurant: UncheckedAccount<'info>,

    pub token_mint: Box<InterfaceAccount<'info, Mint>>,

    #[account(
        init,
        payer = customer,
        token::mint = token_mint,
        token::authority = escrow_vault,
        token::token_program = token_program,
        seeds = [ESCROW_VAULT_SEED, &order_id.to_le_bytes()],
        bump,
    )]
    pub escrow_vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = customer_token_account.owner == customer.key(),
        constraint = customer_token_account.mint == token_mint.key(),
    )]
    pub customer_token_account: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut)]
    pub customer: Signer<'info>,

    /// Optional surge config — if present and active, delivery_amount is scaled up.
    pub surge_config: Option<Box<Account<'info, SurgeConfig>>>,

    pub token_program: Interface<'info, TokenInterface>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<CreateOrder>,
    order_id: u64,
    food_amount: u64,
    delivery_amount: u64,
    initial_contribution: u64,
    code_a_hash: [u8; 32],
    code_b_hash: [u8; 32],
    // estimated_delivery_time: Unix timestamp predicted by the AI routing model (0 = none).
    estimated_delivery_time: i64,
    // ai_confidence: AI routing confidence 0-100 (0 = no AI routing applied).
    ai_confidence: u8,
    // requested_delivery_time: Customer-preferred delivery time (0 = ASAP).
    requested_delivery_time: i64,
    // requested_pickup_time: Customer-preferred pickup time (0 = ASAP).
    requested_pickup_time: i64,
) -> Result<()> {
    let config = &ctx.accounts.protocol_config;
    let mint_key = ctx.accounts.token_mint.key();

    require!(config.is_mint_accepted(&mint_key), ForkitError::UnsupportedMint);
    require!(ai_confidence <= 100, ForkitError::InvalidAIConfidence);

    // Apply surge pricing if the surge_config account is provided and active
    let (effective_delivery_amount, surge_applied) =
        if let Some(surge) = &ctx.accounts.surge_config {
            let surged = surge
                .apply_surge(delivery_amount)
                .ok_or(ForkitError::ArithmeticOverflow)?;
            (surged, surge.active && surge.multiplier_bps > 0)
        } else {
            (delivery_amount, false)
        };

    // Calculate amounts (use surge-adjusted delivery fee)
    let total = food_amount
        .checked_add(effective_delivery_amount)
        .ok_or(ForkitError::ArithmeticOverflow)?;
    let protocol_fee = total
        .checked_mul(config.fee_basis_points as u64)
        .ok_or(ForkitError::ArithmeticOverflow)?
        .checked_div(10000)
        .ok_or(ForkitError::ArithmeticOverflow)?;

    // Total needed in escrow: food + delivery (no deposit)
    let escrow_target = total;

    let clock = Clock::get()?;

    // Initialize order
    let order = &mut ctx.accounts.order;
    order.order_id = order_id;
    order.customer = ctx.accounts.customer.key();
    order.restaurant = ctx.accounts.restaurant.key();
    order.driver = Pubkey::default();
    order.token_mint = mint_key;
    order.food_amount = food_amount;
    order.delivery_amount = effective_delivery_amount;
    order.protocol_fee = protocol_fee;
    order.escrow_target = escrow_target;
    order.escrow_funded = 0;
    order.contributor_count = 0;
    order.code_a_hash = code_a_hash;
    order.code_b_hash = code_b_hash;
    order.status = OrderStatus::Created;
    order.created_at = clock.unix_timestamp;
    order.cancel_deadline = clock.unix_timestamp + CANCEL_WINDOW_SECONDS;
    order.pickup_confirmed_at = 0;
    order.delivery_confirmed_at = 0;
    order.estimated_delivery_time = estimated_delivery_time;
    order.requested_delivery_time = requested_delivery_time;
    order.requested_pickup_time = requested_pickup_time;
    order.ai_confidence = ai_confidence;
    order.bump = ctx.bumps.order;

    // Initialize contribution record for the creator
    let contribution = &mut ctx.accounts.contribution;
    contribution.order_id = order_id;
    contribution.contributor = ctx.accounts.customer.key();
    contribution.amount = 0;
    contribution.bump = ctx.bumps.contribution;

    // If initial contribution > 0, fund immediately
    if initial_contribution > 0 {
        // Cap at escrow target
        let actual_contribution = initial_contribution.min(escrow_target);

        token_interface::transfer_checked(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                TransferChecked {
                    from: ctx.accounts.customer_token_account.to_account_info(),
                    mint: ctx.accounts.token_mint.to_account_info(),
                    to: ctx.accounts.escrow_vault.to_account_info(),
                    authority: ctx.accounts.customer.to_account_info(),
                },
            ),
            actual_contribution,
            ctx.accounts.token_mint.decimals,
        )?;

        contribution.amount = actual_contribution;
        order.escrow_funded = actual_contribution;
        order.contributor_count = 1;

        // Check if fully funded in one go
        if order.is_fully_funded() {
            order.status = OrderStatus::Funded;
            emit!(OrderFunded {
                order_id,
                total_funded: order.escrow_funded,
                contributor_count: order.contributor_count,
            });
        }

        emit!(ContributionMade {
            order_id,
            contributor: ctx.accounts.customer.key(),
            amount: actual_contribution,
            total_funded: order.escrow_funded,
            escrow_target,
            fully_funded: order.is_fully_funded(),
        });
    }

    emit!(OrderCreated {
        order_id,
        customer: order.customer,
        restaurant: order.restaurant,
        token_mint: mint_key,
        food_amount,
        delivery_amount: effective_delivery_amount,
        escrow_target,
        protocol_fee,
        requested_delivery_time,
        requested_pickup_time,
    });

    if estimated_delivery_time > 0 || ai_confidence > 0 || surge_applied {
        emit!(OrderCreatedWithAI {
            order_id,
            estimated_delivery_time,
            ai_confidence,
            surge_applied,
            effective_delivery_amount,
        });
    }

    Ok(())
}
