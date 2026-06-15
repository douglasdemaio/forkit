# 🍴 ForkIt

**Solana smart contract protocol for local commerce — on-chain.**

ForkIt replaces centralized delivery platforms with an open on-chain protocol where any local merchant — restaurants, home kitchens, bookshops, florists, hardware stores — transacts directly with drivers and customers. Payments are held in escrow as SPL tokens (USDC, EURC), verified with delivery codes, and settled automatically — no middleman taking 30%.

## Architecture

ForkIt consists of three Anchor programs deployed to Solana:

| Program | Description | Program ID (devnet) |
|---|---|---|
| **forkit_escrow** | Order lifecycle, escrow, payments, disputes, surge pricing | `CNUWqYhXPXszPuB8psqG2VSnwCXf1MWzT4Pztp4y8fgj` |
| **forkit_registry** | User registration, ratings, reputation, trust scores | `EM1FgSzfS3F7cCYJWhUaqqPAK7ijZYpYRx7pzYkuyExz` |
| **forkit_loyalty** | Points economy, tier progression, protocol-fee discounts | `BnnUntqkUadZ2BsW8j675P9hJQV3aqVcmt4xG4xfeoM8` |
| **forkit_token** | FORKIT SPL token, dynamic minting, reserve basket, annual governance | `FRKTkNq5sGhCVr3QLUwMGaznMaCvGkzNFbPmHd6XbC4q` |

```
forkit/
├── programs/
│   ├── forkit_escrow/      # Order lifecycle, escrow, payments, disputes, surge pricing
│   ├── forkit_registry/    # User registration, ratings, reputation
│   ├── forkit_loyalty/     # Points earning & redemption (Bronze → Platinum)
│   └── forkit_token/       # FORKIT SPL token, reserve basket, governance
├── Anchor.toml             # Anchor workspace config
├── Cargo.toml              # Rust workspace
└── package.json            # Test dependencies
```

---

## Protocol Parameters

| Parameter | Value | Notes |
|---|---|---|
| **Protocol fee** | 0.02% (2 basis points) | Collected on settlement into treasury |
| **Max contributors** | 10 per order | Multi-sig funding support |
| **Max accepted mints** | 20 | Whitelisted SPL tokens |
| **Max surge multiplier** | 3× (30,000 bps) | AI-driven dynamic pricing |
| **Treasury wallet** | `BiP5PJuUiXPYCFx98RMCGCnRhdUVrkxSke9C6y2ZohQ9` | Protocol fee recipient |

### Timeouts

| Phase | Duration |
|---|---|
| Cancel window | 60 seconds |
| Funding window | 15 minutes |
| Preparation | 45 minutes |
| Pickup | 45 minutes |
| Delivery | 3 hours |

### Scheduled Orders

Customers can specify a **requested delivery time** and/or **requested pickup time** when placing an order. These are stored on-chain as Unix timestamps (0 = ASAP). Merchants and drivers use these to plan preparation and routing. The timeout clock still applies from the order creation time to protect all parties.

---

## Order Lifecycle

```
Created → Funded → Preparing → ReadyForPickup → PickedUp → Delivered → Settled
   │         │         │                                        │
   │         │         │                                        └── Disputed → Resolved
   │         │         │
   │         └─────────┼─── (friends contribute for reimbursement)
   │         │
   │         └── (cancel within 60s) → Cancelled
   │
   └── (funding timeout 15min) → Refunded
```

1. **Created** - Customer places an order; funds (goods + delivery fee) are locked in an escrow PDA. If the initial contribution doesn't cover the full amount, others can chip in.
2. **Funded** - Escrow is fully funded. Ready for the merchant. Friends can still contribute to reimburse the original payer.
3. **Preparing** - Merchant (restaurant, home cook, or shop) accepts the order via `accept_order`.
4. **ReadyForPickup** - Merchant marks the order ready.
5. **PickedUp** - Driver confirms pickup by submitting Code A (hash-verified on-chain).
6. **Delivered** - Customer confirms delivery by submitting Code B. Settlement occurs atomically:
   - Merchant receives the goods amount
   - Driver receives the delivery fee
   - Treasury receives the 0.02% protocol fee
7. **Settled** - All parties have claimed their funds. If the escrow was overfunded (friends contributed after funding), excess is returned proportionally to contributors.

Timeouts at any stage trigger automatic refunds. Disputes can be opened after pickup and are resolved by admin arbitration (refund customer, pay merchant+driver, or split).

> **Off-chain API alignment:** The `forkit-site` Next.js API and the `forkme` mobile app use these same status names exactly (PascalCase). Status transitions in the off-chain DB mirror on-chain state — see the forkit-site repository for the full API contract.

### Reimbursement Model

When a customer places an order, they typically front the full amount. Friends can then contribute via `contribute_to_order` - even after the order is funded. These additional contributions are held in the escrow vault. After settlement, the original payer (and any over-contributors) can call `claim_deposit` to receive their proportional share of the excess funds. This effectively lets friends split the bill without requiring coordination upfront.

---

## Programs

### forkit_escrow

Core program managing the full order lifecycle, escrow vault, and fee distribution.

| Instruction | Signer | Description |
|---|---|---|
| `initialize_protocol` | Admin | Set treasury wallet, fee rate, accepted mints |
| `update_protocol_config` | Admin | Change treasury wallet or fee rate |
| `add_accepted_mint` | Admin | Whitelist an SPL token (USDC, EURC, etc.) |
| `set_surge_pricing` | Admin | Enable/disable dynamic surge multiplier (up to 3×) |
| `create_order` | Customer | Create order, lock funds into escrow PDA (with optional scheduled delivery/pickup times) |
| `contribute_to_order` | Anyone | Add funds to an order (up to 10 contributors); accepted before and after funding for reimbursement |
| `accept_order` | Driver | Claim a delivery |
| `cancel_order` | Customer | Cancel within 60s window, triggers full refund |
| `mark_ready_for_pickup` | Merchant | Signal the order is ready |
| `confirm_pickup` | Driver | Verify Code A hash - proves pickup |
| `confirm_delivery` | Customer | Verify Code B hash - triggers settlement + fee distribution |
| `claim_deposit` | Contributor | Claim proportional reimbursement of excess contributions after settlement |
| `refund_contributor` | Anyone | Permissionless refund per contributor after cancel/timeout |
| `timeout_refund` | Anyone | Auto-refund if prep/pickup/delivery times out |
| `open_dispute` | Customer | Escalate after pickup |
| `resolve_dispute` | Admin | Refund customer, pay merchant+driver, or split |

#### Accounts

- **ProtocolConfig** - Global protocol settings (admin, treasury, fee rate, accepted mints)
- **Order** - Per-order state (amounts, status, timestamps, delivery codes, AI routing data, scheduled delivery/pickup times)
- **Contribution** - Per-contributor funding record for an order
- **SurgeConfig** - Dynamic surge pricing state (multiplier, active flag)

#### Events

`OrderCreated` · `OrderFunded` · `ContributionMade` · `OrderAccepted` · `OrderCancelled` · `OrderReadyForPickup` · `PickupConfirmed` · `DeliveryConfirmed` · `ContributorRefunded` · `ContributorReimbursed` · `OrderRefunded` · `DisputeOpened` · `DisputeResolved` · `SurgeUpdated` · `OrderCreatedWithAI`

### forkit_registry

On-chain identity and reputation for all participants.

| Instruction | Signer | Description |
|---|---|---|
| `register` | Anyone | Create a profile PDA (role: Merchant, Driver, or Customer) |
| `update_metadata` | Profile owner | Update profile metadata URI |
| `update_payout_wallet` | Profile owner | Change the payout wallet address (emits auditable on-chain event) |
| `rate_counterparty` | Post-order | Submit 1-5 star rating, recalculates trust score |
| `update_loyalty_points` | Loyalty program | Sync points balance to profile |

#### Accounts

- **Profile** — Per-wallet identity (role, payout wallet, trust score, completed orders, ratings, loyalty points, metadata URI)

#### Trust Score

Trust scores (0-100.00) are calculated from:
- **Base**: Average star rating mapped to 0-100
- **Dispute penalty**: Proportional to disputes lost vs completed orders
- **Inactivity decay**: 1% per day after 30 days of inactivity

#### Events

`ProfileRegistered` · `ProfileRated` · `PayoutWalletChanged`

### forkit_token

FORKIT SPL token creation, dynamic fee routing, batched minting, reserve management, and on-chain governance.

| Instruction | Signer | Description |
|---|---|---|
| `initialize` | Admin | Create FORKIT mint (PDA, no freeze authority), TokenConfig, ReserveConfig, USDC reserve vault |
| `collect_fee` | Customer / escrow CPI | Collect 0.02% protocol fee; split 4 ways; record FORKIT mint obligation for customer |
| `execute_mint_batch` | Anyone | Mint pending FORKIT for one customer once 100 obligations or 1 hour elapsed |
| `update_oracle_price` | Oracle updater | Push hourly spot price; program stores 8-sample TWAP buffer (rate-limited to 1 update/hour) |
| `convert_reserve` | Anyone | Check TWAP oracle, compute reserve allocation drift, emit ReserveUpdated for keepers |
| `open_voting` | Anyone | Open Oct 31 – Nov 7 governance vote with a proposed basket allocation |
| `cast_vote` | FORKIT holder | Vote approve/reject; weight = FORKIT balance at cast time |
| `finalize_vote` | Anyone | Tally after Nov 7 23:59 UTC; apply allocation effective Jan 1 if quorum + majority met |
| `propose_rate_change` | Owner | Propose new FORKIT mint rate; starts 7-day timelock |
| `execute_rate_change` | Anyone | Apply proposed rate after 7-day timelock |
| `request_withdrawal` | Multisig signer | Create reserve withdrawal request (nonce-keyed) |
| `approve_withdrawal` | Multisig signer | Add bitmask approval; 3-of-5 required |
| `execute_withdrawal` | Anyone | Transfer from reserve vault once 3 approvals set |

#### Fee Distribution (0.02% = 200 ppm)

| Split | Destination | Amount |
|---|---|---|
| Platform | `9iBQEn9yMbKVhJKEpMpPByS6pjydPmQDGaznMaCvGkzD` (hardcoded) | 0.005% USDC |
| Customer | FORKIT minted to customer's ATA | 0.005% → FORKIT @ 1/\$0.01 rate |
| Merchant | Merchant's USDC ATA | 0.005% USDC |
| Reserve | Program-owned USDC vault (→ basket) | 0.005% USDC |

All four splits are validated to sum to exactly 0.02% before any transfer executes.

#### FORKIT Mint Rate

`1 FORKIT per $0.01 of fee value` — i.e. a $100 order generates $0.005 in customer fee → **0.5 FORKIT minted**.

The rate is owner-adjustable with a mandatory 7-day timelock. Minting is batched: obligations accumulate per-user and execute in bulk every 100 transactions **or** every 1 hour.

#### Reserve Basket

Target allocation (adjustable via governance): **50% USDC / 30% wSOL / 20% wBTC**.

All reserve valuations use an 8-hour rolling TWAP (8 hourly samples) — never spot price. Rebalancing is triggered when any asset drifts >5% from target. The actual DEX swap (Jupiter CPI) is an integration point left for production deployment.

Withdrawals require **3-of-5 multisig** approval from the registered signers set at initialization.

#### Annual Governance Vote

| Phase | Window |
|---|---|
| Vote open | Oct 31 00:00 UTC |
| Vote close | Nov 7 23:59 UTC |
| New allocations effective | Jan 1 (following year) |

Quorum: ≥10% of circulating FORKIT supply must participate. Majority: >50% approve votes. Supermajority (60%) required if any single asset allocation exceeds 60%.

#### Accounts

- **TokenConfig** — Global config (mint, owner, mint rate, pending rate, timelock, total supply, batch state)
- **ReserveConfig** — Basket allocations, pending governance allocations, multisig signers, withdrawal nonce
- **MintObligation** — Per-customer pending FORKIT mint accumulator
- **OraclePrice** — Per-symbol 8-sample TWAP buffer (SOL/USD, BTC/USD)
- **GovernanceState** — Per-year vote state (proposal, tallies, snapshot supply)
- **VoteRecord** — Per-voter per-year record
- **WithdrawalRequest** — Per-withdrawal multisig approval tracker

#### Events

`ForkitTokenInitialized` · `FeesCollected` · `TokensMinted` · `OraclePriceUpdated` · `ReserveUpdated` · `VotingOpened` · `VoteCast` · `GovernanceVoteResult` · `MintRateChangeProposed` · `MintRateChanged` · `WithdrawalRequested` · `WithdrawalApproved` · `WithdrawalExecuted`

#### Integration with forkit_escrow

`confirm_delivery` in `forkit_escrow` should CPI into `forkit_token::collect_fee` instead of transferring the protocol fee directly to treasury. The escrow vault PDA passes as `source_authority` via `CpiContext::new_with_signer`.

---

### forkit_loyalty

Points economy with tiered protocol-fee discounts.

| Instruction | Signer | Description |
|---|---|---|
| `initialize_loyalty` | Admin | Bind loyalty program to escrow authority |
| `earn_points` | Escrow authority | Award points (1% of order value); +50% bonus for AI-routed orders |
| `redeem_points` | User | Burn points for order discounts |

#### Tier System

| Tier | Lifetime Points | Protocol-Fee Discount |
|---|---|---|
| None | 0 - 499 | 0% |
| Bronze | 500 - 2,499 | 5% |
| Silver | 2,500 - 9,999 | 10% |
| Gold | 10,000 - 49,999 | 15% |
| Platinum | 50,000+ | 20% |

#### Accounts

- **LoyaltyConfig** - Global config (admin, authorized escrow, total points issued)
- **LoyaltyAccount** - Per-user balance, lifetime stats, tier, AI order count

#### Events

`PointsEarned` · `PointsRedeemed` · `TierUpgraded`

---

## Frontend Integration

This section documents the exact PDA derivation and instruction layouts that frontend clients (`forkit-site`, `forkme`) must use when building raw `TransactionInstruction` objects.

### Order ID

The smart contract uses a `u64` for `order_id`. Frontends derive it deterministically from the database UUID by taking the first 16 hex characters (8 bytes) of the UUID and interpreting them as a big-endian u64:

```typescript
function uuidToOrderId(uuid: string): bigint {
  return BigInt('0x' + uuid.replace(/-/g, '').slice(0, 16));
}
```

### PDA Seeds

| Account | Seeds | Notes |
|---|---|---|
| **Order** | `["order", order_id_le_bytes]` | 8-byte little-endian u64 |
| **Escrow Vault** | `["escrow_vault", order_id_le_bytes]` | SPL token account |
| **Contribution** | `["contribution", order_id_le_bytes, contributor_pubkey]` | Per-contributor record |
| **Protocol Config** | `["protocol_config"]` | Global settings |
| **SurgeConfig** | `["surge_config"]` | Optional; pass program ID as sentinel for None |

### Instruction Discriminators (Anchor SHA256 8-byte prefix)

| Instruction | Discriminator |
|---|---|
| `create_order` | `[141, 54, 37, 207, 237, 210, 250, 215]` |
| `contribute_to_order` | `[82, 48, 204, 145, 137, 43, 194, 101]` |
| `confirm_delivery` | `[104, 87, 191, 49, 195, 225, 56, 139]` |

### `create_order` Instruction Data (129 bytes)

```
[0-7]    discriminator (8 bytes)
[8-15]   order_id u64 LE
[16-23]  food_amount u64 LE
[24-31]  delivery_amount u64 LE
[32-39]  initial_contribution u64 LE
[40-71]  code_a_hash [u8; 32]  (SHA256 of code A)
[72-103] code_b_hash [u8; 32]  (SHA256 of code B)
[104-111] estimated_delivery_time i64 LE  (0 = none)
[112]    ai_confidence u8        (0 = no AI routing)
[113-120] requested_delivery_time i64 LE  (0 = ASAP)
[121-128] requested_pickup_time i64 LE    (0 = ASAP)
```

`code_a_hash` and `code_b_hash` are the raw SHA256 bytes of the delivery codes generated by the `forkit-site` API.

### `create_order` Account Order

Must match the `CreateOrder` struct exactly:

1. `order` PDA — init, writable
2. `contribution` PDA — init, writable
3. `protocol_config` PDA — read-only
4. `restaurant` — read-only (`UncheckedAccount`)
5. `token_mint` — read-only
6. `escrow_vault` PDA — init, writable
7. `customer_token_account` — writable
8. `customer` — signer, writable
9. `surge_config` — pass program ID to signal `None`
10. `token_program`
11. `system_program`
12. `rent` sysvar

### `contribute_to_order` Instruction Data (16 bytes)

```
[0-7]  discriminator
[8-15] amount u64 LE
```

Account order: `order`, `contribution`, `token_mint`, `escrow_vault`, `contributor_token_account`, `contributor` (signer), `token_program`, `system_program`, `rent`.

### `confirm_delivery` Instruction Data

```
[0-7]   discriminator
[8-11]  code_b string length u32 LE
[12..]  code_b UTF-8 bytes
```

Pass the **raw** code B string (not the hash). The contract hashes it internally and compares to the stored `code_b_hash`.

Account order: `order`, `escrow_vault`, `protocol_config`, `restaurant_token_account`, `driver_token_account`, `treasury_token_account`, `customer` (signer), `token_program`.

---

## Supported Payment Tokens

| Token | Network | Mint Address |
|---|---|---|
| **USDC** | Devnet | `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU` |
| **USDC** | Mainnet | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` |
| **PYUSD** | Devnet | `CXk2AMBfi3TwaEL2468s6zP8xq9NxTXjp9gjMgzeUynM` |
| **EURC** | Mainnet | `HzwqbKZw8HxMN6bF2yFZNrht3c2iXXzpKcFu7uBEDKtr` (no devnet equivalent) |

Merchants can whitelist accepted mints via `add_accepted_mint`.

---

## Prerequisites

- **Rust** (latest stable) - [rustup.rs](https://rustup.rs)
- **Solana CLI** - [docs.solana.com](https://docs.solana.com/cli/install-solana-cli-tools)
- **Anchor CLI** ≥ 0.30 - [anchor-lang.com](https://www.anchor-lang.com/docs/installation)

```bash
# Generate a devnet keypair if you don't have one
solana-keygen new
solana config set --url devnet
```

---

## Build & Deploy

```bash
# Install test dependencies
npm install

# Build all programs
anchor build

# Deploy to devnet
anchor deploy

# Run tests
anchor test
```

After deployment, update the program IDs in `Anchor.toml` and `.env`, then call `initialize_protocol` with your treasury wallet address to activate fee collection.

---

## Backend & Frontend (packages/)

The `packages/` directory contains an Express API server (`packages/backend/`) and a React/Next.js frontend (`packages/frontend/`).

### Backend — Express API (`packages/backend/`)

The server provides REST endpoints grouped by role. All API routes sit behind the `/api` prefix.

```
packages/backend/
├── src/
│   ├── index.ts                  # Express app entry point
│   ├── api/
│   │   ├── auth.ts               # Wallet-based auth (nonce + signature verify)
│   │   ├── restaurants.ts        # Restaurant registration, menu CRUD
│   │   ├── customers.ts          # Customer orders, cancellation
│   │   ├── drivers.ts            # Driver registration, order acceptance, GPS
│   │   ├── contributions.ts      # Crowdfunded contributions, share links
│   │   └── admin.ts              # Admin dispute resolution
│   ├── middleware/
│   │   └── wallet-auth.ts        # JWT generation, Solana sig verification, auth middleware
│   ├── config/
│   │   └── constants.ts          # Shared protocol constants (fees, RPC URL, image limits)
│   └── services/
│       ├── notification.ts       # Socket.IO real-time events
│       ├── image-storage.ts      # IPFS upload via Pinata
│       └── code-generator.ts     # Deterministic order pickup codes (A/B) + SHA-256 hashing
├── prisma/
│   └── schema.prisma             # PostgreSQL schema (6 models)
├── package.json
└── tsconfig.json
```

**Authentication flow:**
1. Client calls `POST /api/auth/nonce` with their wallet address → receives a message to sign
2. Client signs the message with Phantom/wallet adapter
3. Client calls `POST /api/auth/verify` with `{ walletAddress, signature, message }` → receives a JWT
4. Subsequent requests include `Authorization: Bearer <token>` header

Wallet signature verification uses `tweetnacl` with the Ed25519 curve. Tokens expire after 24 hours.

**Endpoints:**

| Group | Route | Auth | Description |
|---|---|---|---|
| Auth | `POST /api/auth/nonce` | No | Request a nonce for wallet signing |
| Auth | `POST /api/auth/verify` | No | Verify signed nonce, return session token |
| Customers | `POST /api/customers/register` | JWT | Register a customer profile |
| Customers | `POST /api/customers/orders` | JWT | Create an order with escrow params |
| Customers | `POST /api/customers/orders/:id/cancel` | JWT | Cancel an order (within 60s window) |
| Customers | `GET /api/customers/orders` | JWT | Order history |
| Customers | `GET /api/customers/restaurants` | No | Browse restaurants |
| Drivers | `POST /api/drivers/register` | JWT | Register a driver profile |
| Drivers | `PUT /api/drivers/zones` | JWT | Update delivery zones |
| Drivers | `GET /api/drivers/available-orders` | JWT | List `Funded` orders available for pickup |
| Drivers | `POST /api/drivers/orders/:id/accept` | JWT | Accept a delivery |
| Drivers | `PUT /api/drivers/orders/:id/location` | JWT | Push GPS; emits Socket.IO `order:driver-location` |
| Restaurants | `POST /api/restaurants/register` | JWT | Register a restaurant profile |
| Restaurants | `PUT /api/restaurants/profile` | JWT | Update profile (whitelisted fields) |
| Restaurants | `POST /api/restaurants/menu` | JWT | Add/update menu item (optional image) |
| Restaurants | `GET /api/restaurants/menu/:id` | No | Fetch a restaurant's menu |
| Restaurants | `GET /api/restaurants` | No | Browse all restaurants |
| Restaurants | `GET /api/restaurants/orders` | JWT | Fetch restaurant's orders |
| Contributions | `GET /api/contributions/order/:orderId` | No | Funding progress for an order |
| Contributions | `POST /api/contributions` | No | Record an on-chain contribution |
| Contributions | `POST /api/contributions/generate-link/:orderId` | No | Generate a shareable funding link |
| Contributions | `GET /api/contributions/share/:shareLink` | No | Order details from share link |
| Admin | `GET /api/admin/disputes` | JWT+Admin | List disputed orders |
| Admin | `PATCH /api/admin/disputes/:orderId/resolve` | JWT+Admin | Mirror on-chain resolution to Postgres |

#### Prisma Schema

Six models in `packages/backend/prisma/schema.prisma`:

| Model | Key Fields | Relationships |
|---|---|---|
| `Restaurant` | `walletAddress` (unique), `name`, `metadata` (JSON), `acceptedMints` | Has many `MenuItem`, has many `Order` |
| `MenuItem` | `name`, `price`, `imageCid`, `category`, `available` | Belongs to `Restaurant` |
| `Customer` | `walletAddress` (unique), `metadata` (JSON) | Has many `Order` |
| `Driver` | `walletAddress` (unique), `metadata` (JSON), `zones` (JSON) | Has many `Order` |
| `Order` | `onChainOrderId` (BigInt, unique), `status`, `items` (JSON), `escrowTarget`, `escrowFunded` | Belongs to `Customer`, `Restaurant`, optional `Driver`; has many `Contribution` |
| `Contribution` | `walletAddress`, `amount`, `txSignature` | Belongs to `Order`; unique on `[orderId, walletAddress]` |

#### Run the backend

```bash
# Start PostgreSQL
docker compose up -d

# Install deps & push schema
cd packages/backend
npm install
npx prisma db push

# Start dev server (hot reload)
npm run dev

# Prisma Studio (database GUI on :5555)
npx prisma studio
```

### Frontend — Admin Disputes (`packages/frontend/`)

The `/admin/disputes` page lets admins review disputed orders. It shows the contributor breakdown with token-symbol-aware display (USDC, EURC, and devnet variants). Three resolution buttons — **Refund Customer**, **Pay Restaurant & Driver**, and **Split** — build and submit the `resolve_dispute` Anchor instruction on-chain, then call `PATCH /api/admin/disputes/:orderId/resolve` to sync the result to Postgres.

### Constants (`packages/frontend/lib/constants.ts`)

Exports: `TREASURY_WALLET`, `SOLANA_RPC_URL`, `SOLANA_NETWORK`, `API_URL`, `KNOWN_MINTS`, `ESCROW_PROGRAM_ID`, `REGISTRY_PROGRAM_ID`, `LOYALTY_PROGRAM_ID`, `DEVNET_USDC_MINT`, `DEVNET_PYUSD_MINT`.

---

## Security Audit

The following vulnerabilities were identified and patched in the Anchor programs:

### forkit_escrow

| Location | Issue | Fix |
|---|---|---|
| `confirm_delivery.rs` | Unchecked arithmetic — `protocol_fee * food_amount` overflows u64 at large order sizes | All intermediate fee calculations use u128; final values cast to u64 |
| `confirm_delivery.rs` | Missing mint constraints on restaurant/driver/treasury token accounts — attacker could drain escrow to wrong token account | Added `constraint = *.mint == order.token_mint` on all three payout accounts |
| `resolve_dispute.rs` | Unchecked `food_amount + delivery_amount` and `half * food_amount / total` in Split path | u128 intermediates with `checked_mul` / `checked_div` |
| `resolve_dispute.rs` | `PayRestaurantAndDriver` and `Split` paths do not guard against unassigned driver (`order.driver == Pubkey::default()`) | Added `require!(order.driver != Pubkey::default(), DriverNotAssigned)` in both arms |
| `resolve_dispute.rs` | Missing error codes and mint constraints on restaurant/driver token accounts | Added `@ ForkitError::Unauthorized` and `@ ForkitError::UnsupportedMint` to both accounts |
| `accept_order.rs` | No on-chain validation that the signer is a registered, active driver with sufficient reputation | Cross-program verification against forkit_registry Profile PDA; requires `role == Driver`, `is_active`, `trust_score >= 1000` (10.00) |
| `contribute_to_order.rs` | Funding timeout applied to reimbursement contributions (orders already in Funded/Preparing/ReadyForPickup), blocking friends from chipping in | Timeout check scoped to `Created` status only |

### forkit_registry

| Location | Issue | Fix |
|---|---|---|
| `rate_counterparty.rs` | No PDA seeds constraint on `target_profile` — caller could pass any account that passes the discriminator check | Added `seeds = [Profile::SEED, target_profile.wallet, &[role as u8]]` with `bump = target_profile.bump` |
| `rate_counterparty.rs` | No self-rating prevention — a profile could boost its own trust score | Added `constraint = target_profile.wallet != rater.key() @ RegistryError::SelfRatingNotAllowed` |
| `rate_counterparty.rs` | Unchecked `total_ratings += 1` and `sum_ratings += rating` — overflow at u64 max | Replaced with `checked_add` returning `RegistryError::ArithmeticOverflow` |
| `rate_counterparty.rs` | Generic `ErrorCode::ConstraintRaw` used for invalid rating value | New `errors.rs` module with `RegistryError::{InvalidRating, SelfRatingNotAllowed, ArithmeticOverflow}` |

---

## Related Repositories

| Repo | Description |
|---|---|
| [**forkit-site**](https://github.com/douglasdemaio/forkit-site) | Web frontend for ForkIt |
| [**forkme**](https://github.com/douglasdemaio/forkme) | Mobile app for ForkIt |

---

## License

[MIT with Fork Compensation Clause](LICENSE)
