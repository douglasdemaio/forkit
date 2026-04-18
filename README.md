# 🍴 ForkIt

**Solana smart contract protocol for decentralized food delivery.**

ForkIt replaces centralized delivery platforms with an open on-chain protocol where restaurants, drivers, and customers interact directly. Payments are held in escrow as SPL tokens (USDC, EURC), verified with delivery codes, and settled automatically - no middleman taking 30%.

## Architecture

ForkIt consists of three Anchor programs deployed to Solana:

| Program | Description | Program ID (devnet) |
|---|---|---|
| **forkit_escrow** | Order lifecycle, escrow, payments, disputes, surge pricing | `FNZXjjq2oceq15jVsnHT8gYJQUZ9NLCXCpYak2pXsqGB` |
| **forkit_registry** | User registration, ratings, reputation, trust scores | `2riHMdVB6eFgeQjqvnqq2Mrpqea7hrMv5ZNRh7gZgB9S` |
| **forkit_loyalty** | Points economy, tier progression, protocol-fee discounts | `6DaFmi7haz2Ci9sXaHRviz3biwbmTwipvwc9L9cdeugR` |

```
forkit/
├── programs/
│   ├── forkit_escrow/      # Order lifecycle, escrow, payments, disputes, surge pricing
│   ├── forkit_registry/    # User registration, ratings, reputation
│   └── forkit_loyalty/     # Points earning & redemption (Bronze → Platinum)
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

Customers can specify a **requested delivery time** and/or **requested pickup time** when placing an order. These are stored on-chain as Unix timestamps (0 = ASAP). Restaurants and drivers use these to plan preparation and routing. The timeout clock still applies from the order creation time to protect all parties.

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

1. **Created** — Customer places an order; funds (food + delivery fee) are locked in an escrow PDA. If the initial contribution doesn’t cover the full amount, others can chip in.
2. **Funded** — Escrow is fully funded. Ready for the restaurant. Friends can still contribute to reimburse the original payer.
3. **Preparing** — Restaurant accepts the order via `accept_order`.
4. **ReadyForPickup** — Restaurant marks food as ready.
5. **PickedUp** — Driver confirms pickup by submitting Code A (hash-verified on-chain).
6. **Delivered** — Customer confirms delivery by submitting Code B. Settlement occurs atomically:
   - Restaurant receives the food amount
   - Driver receives the delivery fee
   - Treasury receives the 0.02% protocol fee
7. **Settled** — All parties have claimed their funds. If the escrow was overfunded (friends contributed after funding), excess is returned proportionally to contributors.

Timeouts at any stage trigger automatic refunds. Disputes can be opened after pickup and are resolved by admin arbitration (refund customer, pay restaurant+driver, or split).

### Reimbursement Model

When a customer places an order, they typically front the full amount. Friends can then contribute via `contribute_to_order` — even after the order is funded. These additional contributions are held in the escrow vault. After settlement, the original payer (and any over-contributors) can call `claim_deposit` to receive their proportional share of the excess funds. This effectively lets friends split the bill without requiring coordination upfront.

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
| `mark_ready_for_pickup` | Restaurant | Signal food is ready |
| `confirm_pickup` | Driver | Verify Code A hash - proves pickup |
| `confirm_delivery` | Customer | Verify Code B hash - triggers settlement + fee distribution |
| `claim_deposit` | Contributor | Claim proportional reimbursement of excess contributions after settlement |
| `refund_contributor` | Anyone | Permissionless refund per contributor after cancel/timeout |
| `timeout_refund` | Anyone | Auto-refund if prep/pickup/delivery times out |
| `open_dispute` | Customer | Escalate after pickup |
| `resolve_dispute` | Admin | Refund customer, pay restaurant+driver, or split |

#### Accounts

- **ProtocolConfig** - Global protocol settings (admin, treasury, fee rate, accepted mints)
- **Order** — Per-order state (amounts, status, timestamps, delivery codes, AI routing data, scheduled delivery/pickup times)
- **Contribution** - Per-contributor funding record for an order
- **SurgeConfig** - Dynamic surge pricing state (multiplier, active flag)

#### Events

`OrderCreated` · `OrderFunded` · `ContributionMade` · `OrderAccepted` · `OrderCancelled` · `OrderReadyForPickup` · `PickupConfirmed` · `DeliveryConfirmed` · `ContributorRefunded` · `ContributorReimbursed` · `OrderRefunded` · `DisputeOpened` · `DisputeResolved` · `SurgeUpdated` · `OrderCreatedWithAI`

### forkit_registry

On-chain identity and reputation for all participants.

| Instruction | Signer | Description |
|---|---|---|
| `register` | Anyone | Create a profile PDA (role: Restaurant, Driver, or Customer) |
| `update_metadata` | Profile owner | Update profile metadata URI |
| `rate_counterparty` | Post-order | Submit 1-5 star rating, recalculates trust score |
| `update_loyalty_points` | Loyalty program | Sync points balance to profile |

#### Accounts

- **Profile** - Per-wallet identity (role, trust score, completed orders, ratings, loyalty points, metadata URI)

#### Trust Score

Trust scores (0-100.00) are calculated from:
- **Base**: Average star rating mapped to 0-100
- **Dispute penalty**: Proportional to disputes lost vs completed orders
- **Inactivity decay**: 1% per day after 30 days of inactivity

#### Events

`ProfileRegistered` · `ProfileRated`

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

## Supported Payment Tokens

| Token | Network | Mint Address |
|---|---|---|
| **USDC** | Devnet | `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU` |
| **USDC** | Mainnet | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` |
| **EURC** | Devnet | `CXk2AMBfi3TwaEL2468s6zP8xq9NxTXjp9gjMgzeUynM` |
| **EURC** | Mainnet | `HzwqbKZw8HxMN6bF2yFZNrht3c2iXXzpKcFu7uBEDKtr` |

Restaurants can whitelist accepted mints via `add_accepted_mint`.

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

## Related Repositories

| Repo | Description |
|---|---|
| [**forkit-site**](https://github.com/douglasdemaio/forkit-site) | Web frontend for ForkIt |
| [**forkme**](https://github.com/douglasdemaio/forkme) | Mobile app for ForkIt |

---

## License

[MIT with Fork Compensation Clause](LICENSE)
