# 🍴 ForkIt

**Decentralized food delivery protocol on Solana.**

ForkIt replaces centralized delivery platforms with an open protocol where restaurants, drivers, and customers interact directly. Payments are held in on-chain escrow, verified with delivery codes, and settled automatically — no middleman taking 30%.

## How It Works

1. **Customer** places an order → escrow account created, anyone can contribute funds
2. **Friends chip in** → share a link, friends send tokens directly to the smart contract
3. **Order funded** → restaurant sees the order, starts preparing
4. **Driver** picks up (verified with Code A) → delivers (verified with Code B)
5. **Settlement** — restaurant and driver paid automatically; deposits returned proportionally to all contributors

Disputes go to admin arbitration. Timeouts trigger automatic refunds. Protocol fee is 0.02%.

## 💸 Split Payments — The Key Feature

Anyone can contribute funds to any order. The smart contract is the source of truth — funds go directly to it, not through the app. This means:

- **Friends can help pay** — share a link, they contribute from their wallet
- **Strangers can tip** — send tokens to any open order
- **The app is separate from the funds** — the contract manages everything

### Example: Pizza Night 🍕

A group orders 2 pizzas:
- Food: **18 USDC**
- Delivery: **2 USDC**
- Order total: **20 USDC**
- Deposit (2× multiplier): **40 USDC**
- **Escrow target: 60 USDC** (order + deposit)

**Person 1** (order creator) sends **42 USDC** (70% of escrow):
```
create_order(food=18, delivery=2, initial_contribution=42)
→ Order created, 42/60 USDC funded (70%)
→ Status: Created (awaiting remaining funds)
```

**Person 2** (friend) sends **18 USDC** (30% of escrow):
```
contribute_to_order(amount=18)
→ Order fully funded at 60/60 USDC
→ Status: Funded ✅ (ready for restaurant)
```

**After successful delivery:**
- Restaurant gets: 18 USDC (minus 0.02% fee)
- Driver gets: 2 USDC (minus 0.02% fee)
- Treasury gets: ~0.004 USDC protocol fee
- **Person 1 claims deposit:** 40 × 42/60 = **28 USDC** back
- **Person 2 claims deposit:** 40 × 18/60 = **12 USDC** back

Each contributor gets their proportional share of the deposit returned. The order creator (Person 1) is the one who receives delivery codes in the app.

### How Contributions Work On-Chain

```
┌─────────────────────────────────────────────────┐
│                  Order PDA                       │
│  escrow_target: 60 USDC                         │
│  escrow_funded: 0 → 42 → 60                     │
│  contributor_count: 0 → 1 → 2                   │
│  status: Created → Created → Funded             │
└─────────────────────────────────────────────────┘
         ↓                    ↓
┌──────────────────┐  ┌──────────────────┐
│ Contribution PDA │  │ Contribution PDA │
│ Person 1: 42     │  │ Person 2: 18     │
└──────────────────┘  └──────────────────┘
         ↓                    ↓
┌─────────────────────────────────────────────────┐
│              Escrow Vault (SPL Token)            │
│  60 USDC total                                   │
└─────────────────────────────────────────────────┘
```

**Funding timeout:** 15 minutes. If the order isn't fully funded, anyone can crank `timeout_refund` and all contributors reclaim their tokens via `refund_contributor`.

**Max contributors:** 10 per order (keeps account sizes bounded).

## Architecture

```
forkit/
├── packages/
│   ├── contracts/       # Solana programs (Anchor/Rust)
│   │   ├── forkit_escrow     # Order lifecycle, split payments, disputes
│   │   ├── forkit_registry   # User registration, ratings, reputation
│   │   └── forkit_loyalty    # Points earning & redemption
│   ├── backend/         # Express + Prisma API server
│   │   ├── src/api/          # REST endpoints (auth, customers, drivers, restaurants, contributions)
│   │   ├── src/services/     # Matching, pricing, notifications, image storage
│   │   └── prisma/           # PostgreSQL schema
│   └── frontend/        # Next.js + Tailwind web app
│       ├── app/              # Pages (landing, restaurant dashboard)
│       ├── components/       # Wallet connect, code input, trust badges
│       └── hooks/            # useEscrow, useOrderStatus, useWalletAuth
├── turbo.json           # Turborepo pipeline config
└── package.json         # Workspace root
```

**Companion mobile app:** [ForkMe](https://github.com/douglasdemaio/forkme) — React Native/Expo app for iOS, Android, and Solana Seeker.

## Prerequisites

- **Node.js** ≥ 18
- **Rust** + [Anchor CLI](https://www.anchor-lang.com/docs/installation) ≥ 0.30
- **Solana CLI** with a devnet keypair (`solana-keygen new`)
- **PostgreSQL** 15+
- **Redis** (for real-time matching/notifications)

## Quick Start

### 1. Clone & Install

```bash
git clone https://github.com/douglasdemaio/forkit.git
cd forkit
npm install
```

### 2. Environment

```bash
cp .env.example .env
```

Edit `.env` with your values:

| Variable | Description |
|---|---|
| `SOLANA_RPC_URL` | Solana RPC endpoint (default: devnet) |
| `DATABASE_URL` | PostgreSQL connection string |
| `REDIS_URL` | Redis connection string |
| `JWT_SECRET` | Secret for auth tokens |
| `PINATA_API_KEY` / `PINATA_SECRET_KEY` | IPFS pinning for menu images |
| `NEXT_PUBLIC_MAPBOX_TOKEN` | Map rendering in frontend |
| `TREASURY_WALLET` | Protocol fee recipient |

### 3. Build & Deploy Contracts

```bash
cd packages/contracts
anchor build
anchor deploy    # deploys to devnet by default
```

After deployment, update the program IDs in `.env` and `Anchor.toml`.

### 4. Set Up Database

```bash
cd packages/backend
npx prisma migrate dev
```

### 5. Run Everything

From the repo root:

```bash
npm run dev
```

This starts both backend (port 3001) and frontend (port 3000) via Turborepo.

## Solana Programs

### Escrow (`forkit_escrow`)

The core program. Manages order lifecycle with **multi-payer split payments**:

| Instruction | Who | What |
|---|---|---|
| `initialize_protocol` | Admin | Set treasury, fee rate, accepted token mints |
| `add_accepted_mint` | Admin | Whitelist an SPL token for payments |
| `create_order` | Customer | Create order + optional initial contribution |
| `contribute_to_order` | Anyone | Send tokens to an order's escrow |
| `cancel_order` | Customer | Cancel within 60s window (sets Cancelled status) |
| `accept_order` | Driver | Claim a funded order for delivery |
| `mark_ready_for_pickup` | Restaurant | Signal food is ready |
| `confirm_pickup` | Driver | Verify Code A hash → proves pickup |
| `confirm_delivery` | Customer | Verify Code B hash → pays restaurant & driver |
| `claim_deposit` | Contributor | Claim proportional deposit share after settlement |
| `refund_contributor` | Anyone | Refund a contributor after cancel/timeout (permissionless crank) |
| `timeout_refund` | Anyone | Mark timed-out order for refunds (permissionless crank) |
| `open_dispute` | Customer | Escalate after pickup |
| `resolve_dispute` | Admin | Refund, pay, or split |

**Order states:** `Created → Funded → Preparing → ReadyForPickup → PickedUp → Delivered → Settled`

**Timeouts:** Funding 15min, Prep 45min, Pickup 30min, Delivery 2hr.

**Accounts:**
- `Order` PDA — `seeds = [b"order", order_id]`
- `Contribution` PDA — `seeds = [b"contribution", order_id, contributor_pubkey]`
- `Escrow Vault` — `seeds = [b"escrow_vault", order_id]`

### Registry (`forkit_registry`)

On-chain identity for all participants:

- `register` — create a profile (wallet-indexed PDA)
- `update_metadata` — update name, location, zones
- `rate_counterparty` — post-order ratings (1-5)
- `update_loyalty_points` — sync points from loyalty program

### Loyalty (`forkit_loyalty`)

Simple points system:

- `earn_points` — awarded after successful deliveries
- `redeem_points` — spend points for discounts

## Backend API

Base URL: `http://localhost:3001`

### Auth
- `POST /api/auth/challenge` — get a nonce to sign
- `POST /api/auth/verify` — submit signed nonce → JWT

All other endpoints require `Authorization: Bearer <jwt>` unless noted.

### Customers
- `GET /api/customers/nearby-restaurants` — list restaurants in range
- `POST /api/customers/orders` — create an order
- `GET /api/customers/orders/:id` — order status

### Restaurants
- `POST /api/restaurants/register` — register with menu
- `PUT /api/restaurants/menu` — update menu items
- `GET /api/restaurants/orders` — incoming orders

### Drivers
- `POST /api/drivers/register` — register with delivery zones
- `GET /api/drivers/available-orders` — orders needing pickup
- `POST /api/drivers/accept/:orderId` — claim a delivery

### Contributions (Split Payments)
- `GET /api/contributions/order/:orderId` — funding progress + all contributions
- `GET /api/contributions/share/:shareLink` — order info via shareable link (public, no auth)
- `POST /api/contributions` — record a contribution (after on-chain tx)
- `POST /api/contributions/generate-link/:orderId` — generate shareable link

### Real-Time

WebSocket via Socket.IO for live order status and funding progress updates.

## Frontend

Next.js app with Solana wallet adapter integration:

- **Wallet Connect** — Phantom, Solflare, etc.
- **Order Flow** — browse → order → share link → track funding → track delivery → confirm
- **Split Payment UI** — share link with friends, see real-time funding progress bar
- **Restaurant Dashboard** — manage menu, view funded orders, mark ready
- **Trust Badges** — on-chain reputation scores displayed per user

## Data Model

```
Restaurant ←─── MenuItem
    │
    ├──── Order ────→ Customer
    │       │
    │       ├──────→ Driver
    │       │
    │       └──── Contribution[] ←── Anyone (friends, strangers)
```

Orders reference on-chain escrow via `onChainOrderId`. Each contribution maps to a `Contribution` PDA on-chain. Off-chain data (menus, metadata, zones) lives in PostgreSQL. Menu images are pinned to IPFS via Pinata.

## Development

```bash
# Run all packages in dev mode
npm run dev

# Build everything
npm run build

# Run tests (contracts)
cd packages/contracts && anchor test

# Database operations
cd packages/backend
npx prisma studio     # GUI for browsing data
npx prisma migrate dev # apply migrations
```

## Related

- **[ForkMe](https://github.com/douglasdemaio/forkme)** — Mobile companion app (iOS, Android, Solana Seeker)

## License

MIT
