use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke_signed, system_instruction};
use anchor_spl::token_interface::{self, TokenInterface, TokenAccount, TransferChecked};
use crate::state::*;
use crate::errors::ForkitError;

/// SPL Token v1 (Tokenkeg) and Token-2022 (Pkqq) initialize_account3 packed-data layout:
/// [0] = u8 instruction discriminator (= 18 for InitializeAccount3)
/// [1..33] = owner pubkey
fn build_initialize_account3_data(owner: &Pubkey) -> Vec<u8> {
    let mut data = Vec::with_capacity(33);
    data.push(18); // InitializeAccount3
    data.extend_from_slice(owner.as_ref());
    data
}

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

    /// Token mint. Validated via the SPL `transfer_checked` CPI in the handler
    /// (which enforces mint correctness) rather than as `InterfaceAccount<Mint>`,
    /// to keep `try_accounts` within the SBF stack-frame budget.
    /// CHECK: validated by SPL token program during transfer
    pub token_mint: UncheckedAccount<'info>,

    /// Escrow vault token account, created manually in the handler so the
    /// CPI to system_program::create_account + token_program::initialize_account3
    /// runs in the handler's stack frame rather than nested inside `try_accounts`.
    /// Using Anchor's `init` for a token account here pushes the CPI 5 frames
    /// deep and overflows the SBF 4KB stack.
    /// CHECK: PDA derivation enforced by `seeds`/`bump`; ownership/authority enforced
    /// by initialize_account3 CPI in the handler.
    #[account(
        mut,
        seeds = [ESCROW_VAULT_SEED, &order_id.to_le_bytes()],
        bump,
    )]
    pub escrow_vault: UncheckedAccount<'info>,

    /// Customer's source token account. Validated by the SPL `transfer_checked` CPI
    /// (which enforces mint+owner+token-program correctness) rather than by Anchor
    /// constraints, to keep `try_accounts` within the SBF stack-frame budget.
    /// CHECK: validated by SPL token program during transfer
    #[account(mut)]
    pub customer_token_account: UncheckedAccount<'info>,

    #[account(mut)]
    pub customer: Signer<'info>,

    /// Optional surge config — if present and active, delivery_amount is scaled up.
    /// Passed as UncheckedAccount (rather than `Option<Box<Account<SurgeConfig>>>`) to keep
    /// `CreateOrder::try_accounts` within the SBF stack-frame budget. The handler
    /// manually deserializes when the account is owned by this program, and treats
    /// any other owner (including the escrow program ID itself, used as a placeholder
    /// for "no surge") as "no surge applied".
    /// CHECK: validated in handler — owner check + manual deserialization
    pub surge_config: UncheckedAccount<'info>,

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

    // Decimals read manually from the mint account (token_mint is UncheckedAccount
    // to keep `try_accounts` within the SBF stack budget). Layout for both SPL Token
    // and Token-2022 mints: bytes [0..36] = mint_authority + supply, byte [44] = decimals.
    let mint_decimals = {
        let data = ctx.accounts.token_mint.try_borrow_data()?;
        require!(data.len() >= 45, ForkitError::UnsupportedMint);
        data[44]
    };

    // Manually create + initialize the escrow vault token account. We can't use
    // Anchor's `init`-with-token-CPI because that nests the CPI 5 frames deep and
    // overflows the SBF 4KB stack. Doing it here keeps the CPI in the handler's
    // own (fresh) frame.
    {
        let order_id_le = order_id.to_le_bytes();
        let bump = [ctx.bumps.escrow_vault];
        let signer_seeds: &[&[u8]] = &[ESCROW_VAULT_SEED, &order_id_le, &bump];

        // Account size is the base SPL token account length (165 bytes), which is
        // accepted by both Tokenkeg and Token-2022 (extensions live beyond this).
        const TOKEN_ACCOUNT_LEN: u64 = 165;
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(TOKEN_ACCOUNT_LEN as usize);

        // 1. Create the bare account, owned by the token program.
        let create_ix = system_instruction::create_account(
            ctx.accounts.customer.key,
            ctx.accounts.escrow_vault.key,
            lamports,
            TOKEN_ACCOUNT_LEN,
            ctx.accounts.token_program.key,
        );
        invoke_signed(
            &create_ix,
            &[
                ctx.accounts.customer.to_account_info(),
                ctx.accounts.escrow_vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            &[signer_seeds],
        )?;

        // 2. InitializeAccount3 — sets mint + authority. Authority = escrow_vault PDA.
        let init_data = build_initialize_account3_data(ctx.accounts.escrow_vault.key);
        let init_ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: *ctx.accounts.token_program.key,
            accounts: vec![
                anchor_lang::solana_program::instruction::AccountMeta::new(*ctx.accounts.escrow_vault.key, false),
                anchor_lang::solana_program::instruction::AccountMeta::new_readonly(*ctx.accounts.token_mint.key, false),
            ],
            data: init_data,
        };
        anchor_lang::solana_program::program::invoke(
            &init_ix,
            &[
                ctx.accounts.escrow_vault.to_account_info(),
                ctx.accounts.token_mint.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
            ],
        )?;
    }

    // Apply surge pricing if the surge_config account is owned by this program
    // and successfully deserializes; otherwise treat as no surge.
    let (effective_delivery_amount, surge_applied) = {
        let surge_ai = &ctx.accounts.surge_config;
        if surge_ai.owner == ctx.program_id {
            let data = surge_ai.try_borrow_data()?;
            let surge = SurgeConfig::try_deserialize(&mut data.as_ref())?;
            let surged = surge
                .apply_surge(delivery_amount)
                .ok_or(ForkitError::ArithmeticOverflow)?;
            (surged, surge.active && surge.multiplier_bps > 0)
        } else {
            (delivery_amount, false)
        }
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
            mint_decimals,
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
