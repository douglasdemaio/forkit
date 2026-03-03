# 🍴 ForkIt

**Decentralized food delivery protocol on Solana.**

ForkIt replaces centralized delivery platforms with an open protocol where restaurants, drivers, and customers interact directly. Payments are held in on-chain escrow, verified with delivery codes, and settled automatically — no middleman taking 30%.

## How It Works

1. **Customer** places an order → funds + 2% deposit locked in escrow
2. **Restaurant** accepts and prepares the food, marks it ready
3. **Driver** picks up (verified with Code A) → delivers (verified with Code B)
4. **Settlement** — restaurant and driver are paid automatically; deposit returned to customer

Disputes go to admin arbitration. Timeouts trigger automatic refunds. Protocol fee is 0.02%. Customer deposit is 2% (returned upon delivery confirmation).

## Architecture

```
forkit/
├── packages/
│   ├── contracts/       # Solana programs (Anchor/Rust)
│   │   ├── forkit_escrow     # Order lifecycle, payments, disputes
│   │   ├── forkit_registry   # User registration, ratings, reputation
│   │   └── forkit_loyalty    # Points earning & redemption
│   ├── backend/         # Express + Prisma API server
│   │   ├── src/api/          # REST endpoints (auth, customers, drivers, restaurants)
│   │   ├── src/services/     # Matching, pricing, notifications, image storage
│   │   └── prisma/           # PostgreSQL schema
│   └── frontend/        # Next.js + Tailwind web app
│       ├── app/              # Pages (landing, restaurant dashboard)
│       ├── components/       # Wallet connect, code input, trust badges
│       └── hooks/            # useEscrow, useOrderStatus, useWalletAuth
├── turbo.json           # Turborepo pipeline config
└── package.json         # Workspace root
```

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

The core program. Manages the full order lifecycle:

| Instruction | Who | What |
|---|---|---|
| `initialize_protocol` | Admin | Set treasury, fee rate, accepted token mints |
| `add_accepted_mint` | Admin | Whitelist an SPL token for payments |
| `create_order` | Customer | Lock funds + 2% deposit into escrow PDA |
| `cancel_order` | Customer | Cancel within 60s window, full refund |
| `mark_ready_for_pickup` | Restaurant | Signal food is ready |
| `accept_order` | Driver | Claim a delivery |
| `confirm_pickup` | Driver | Verify Code A hash → proves pickup happened |
| `confirm_delivery` | Customer | Verify Code B hash → triggers settlement |
| `open_dispute` | Customer | Escalate after pickup if something's wrong |
| `resolve_dispute` | Admin | Refund, pay, or split |
| `timeout_refund` | Anyone | Auto-refund if prep/pickup/delivery times out |

**Timeouts:** Prep 45min, Pickup 30min, Delivery 2hr.

**Order states:** `Created → Preparing → ReadyForPickup → PickedUp → Delivered → Settled`

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

All other endpoints require `Authorization: Bearer <jwt>`.

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

### Real-Time

WebSocket via Socket.IO for live order status updates. Connect with the JWT as auth.

## Frontend

Next.js app with Solana wallet adapter integration:

- **Wallet Connect** — Phantom, Solflare, etc.
- **Order Flow** — browse → order → track → confirm delivery
- **Restaurant Dashboard** — manage menu, view incoming orders, mark ready
- **Trust Badges** — on-chain reputation scores displayed per user

## Data Model

```
Restaurant ←─── MenuItem
    │
    ├──── Order ────→ Customer
    │       │
    │       └──────→ Driver
```

Orders reference on-chain escrow via `onChainOrderId`. Off-chain data (menus, metadata, zones) lives in PostgreSQL. Menu images are pinned to IPFS via Pinata.

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

## License

MIT
