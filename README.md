# 🍴 ForkIt

**Decentralized food delivery protocol on Solana.**

ForkIt replaces centralized delivery platforms with an open protocol where restaurants, drivers, and customers interact directly. Payments are held in on-chain escrow as **USDC or EURC**, verified with delivery codes, and settled automatically — no middleman taking 30%.

## How It Works

1. **Customer** selects USDC or EURC, places an order → funds + 2% deposit locked in escrow
2. **Restaurant** accepts and prepares the food, marks it ready
3. **Driver** (human or AI-routed) picks up (verified with Code A) → delivers (verified with Code B)
4. **Settlement** — restaurant and driver paid automatically on-chain; 2% deposit returned to customer; 0.02% protocol fee sent to treasury

Disputes go to admin arbitration. Timeouts trigger automatic refunds.

---

## Protocol Fee

| Parameter | Value | Where configured |
|---|---|---|
| Rate | **0.02%** (2 basis points) | `packages/backend/src/config/constants.ts` → `FEE_BASIS_POINTS = 2` |
| Treasury wallet | **`BiP5PJuUiXPYCFx98RMCGCnRhdUVrkxSke9C6y2ZohQ9`** | `packages/frontend/lib/constants.ts` → `TREASURY_WALLET` |
| On-chain storage | `ProtocolConfig.treasury_wallet` (PDA) | `packages/contracts/programs/forkit_escrow/src/state.rs:56` |
| On-chain enforcement | `confirm_delivery` instruction verifies treasury token account owner | `packages/contracts/programs/forkit_escrow/src/instructions/confirm_delivery.rs:48` |
| Set/changed by | Admin via `initialize_protocol` + `update_protocol_config` instructions | `packages/contracts/programs/forkit_escrow/src/instructions/initialize_protocol.rs` |

The fee is collected as USDC or EURC (whichever token the order uses) into a token account owned by the treasury wallet. It is transferred atomically during `confirm_delivery` — the same transaction that pays the restaurant and driver.

> **To update the treasury wallet**: call `update_protocol_config` from the admin keypair, then update `TREASURY_WALLET` in both `packages/frontend/lib/constants.ts` and `packages/backend/src/config/constants.ts`.

---

## Supported Payment Tokens

| Token | Network | Mint address |
|---|---|---|
| **USDC** | Devnet | `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU` |
| **USDC** | Mainnet | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` |
| **EURC** | Devnet | `CXk2AMBfi3TwaEL2468s6zP8xq9NxTXjp9gjMgzeUynM` |
| **EURC** | Mainnet | `HzwqbKZw8HxMN6bF2yFZNrht3c2iXXzpKcFu7uBEDKtr` |

Mints are defined in `packages/frontend/lib/constants.ts` → `SUPPORTED_TOKENS`. Restaurants can whitelist accepted mints via `add_accepted_mint` on the escrow program.

---

## Architecture

```
forkit/
├── packages/
│   ├── contracts/       # Solana programs (Anchor/Rust)
│   │   ├── forkit_escrow     # Order lifecycle, payments, disputes, surge pricing
│   │   ├── forkit_registry   # User registration, ratings, reputation
│   │   └── forkit_loyalty    # Points earning & redemption (Bronze → Platinum)
│   ├── backend/         # Express + Prisma API server
│   │   ├── src/api/          # REST endpoints (auth, customers, drivers, restaurants, contributions)
│   │   ├── src/services/     # Matching, pricing, notifications, image storage
│   │   └── prisma/           # PostgreSQL schema
│   └── frontend/        # Next.js + Tailwind web app  (see packages/frontend/README.md)
│       ├── app/              # Pages: landing, restaurant dashboard, order tracking (/order/[id])
│       ├── components/       # Wallet connect, currency selector, order tracker, receipt, code input
│       └── hooks/            # useEscrow, useOrderTracking, useOrderStatus, useWalletAuth
├── turbo.json           # Turborepo pipeline config
└── package.json         # Workspace root
```

---

## Prerequisites

- **Node.js** ≥ 18
- **Rust** + [Anchor CLI](https://www.anchor-lang.com/docs/installation) ≥ 0.30
- **Solana CLI** with a devnet keypair (`solana-keygen new`)
- **PostgreSQL** 15+

---

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

| Variable | Description |
|---|---|
| `SOLANA_RPC_URL` | Solana RPC endpoint (default: devnet) |
| `DATABASE_URL` | PostgreSQL connection string |
| `JWT_SECRET` | Secret for auth tokens |
| `PINATA_API_KEY` / `PINATA_SECRET_KEY` | IPFS pinning for menu images |
| `TREASURY_WALLET` | Protocol fee recipient address (0.02% of each order) |
| `NEXT_PUBLIC_SOLANA_RPC_URL` | Frontend RPC URL |
| `NEXT_PUBLIC_SOLANA_NETWORK` | `devnet` or `mainnet-beta` |
| `NEXT_PUBLIC_API_URL` | Backend URL (default: `http://localhost:3001`) |

### 3. Build & Deploy Contracts

```bash
cd packages/contracts
anchor build
anchor deploy    # deploys to devnet by default
```

After deployment, update the program IDs in `.env` and `Anchor.toml`, then call `initialize_protocol` with your treasury wallet address to activate fee collection.

### 4. Set Up Database

```bash
cd packages/backend
npx prisma migrate dev
```

### 5. Run Everything

```bash
npm run dev
```

Starts backend (port 3001) and frontend (port 3000) via Turborepo.

---

## Solana Programs

All three programs are deployed to Solana devnet:

| Program | ID |
|---|---|
| `forkit_escrow` | `FNZXjjq2oceq15jVsnHT8gYJQUZ9NLCXCpYak2pXsqGB` |
| `forkit_registry` | `2riHMdVB6eFgeQjqvnqq2Mrpqea7hrMv5ZNRh7gZgB9S` |
| `forkit_loyalty` | `6DaFmi7haz2Ci9sXaHRviz3biwbmTwipvwc9L9cdeugR` |

### Escrow (`forkit_escrow`)

Core program. Manages the full order lifecycle, escrow vault, and fee distribution.

| Instruction | Who | What |
|---|---|---|
| `initialize_protocol` | Admin | Set treasury wallet, fee rate (2 bps), accepted mints |
| `update_protocol_config` | Admin | Change treasury wallet or fee rate |
| `add_accepted_mint` | Admin | Whitelist an SPL token (USDC, EURC, etc.) |
| `set_surge_pricing` | Admin | Enable dynamic surge multiplier (up to 3×) |
| `create_order` | Customer | Lock funds + 2% deposit into escrow PDA |
| `contribute_to_order` | Anyone | Multi-sig funding (up to 10 contributors) |
| `cancel_order` | Customer | Cancel within 60s, full refund |
| `mark_ready_for_pickup` | Restaurant | Signal food is ready |
| `accept_order` | Driver | Claim a delivery |
| `confirm_pickup` | Driver | Verify Code A hash → proves pickup |
| `confirm_delivery` | Customer | Verify Code B hash → triggers settlement + fee distribution |
| `claim_deposit` | Customer | Claim 2% deposit refund after settlement |
| `refund_contributor` | Anyone | Permissionless refund per contributor after cancel/timeout |
| `open_dispute` | Customer | Escalate after pickup if something's wrong |
| `resolve_dispute` | Admin | Refund, pay, or split funds |
| `timeout_refund` | Anyone | Auto-refund if prep/pickup/delivery times out |

**Timeouts:** Prep 45 min · Pickup 30 min · Delivery 2 hr · Cancel window 60 s

**Order states:** `Created → Preparing → ReadyForPickup → PickedUp → Delivered → Settled`

**On `confirm_delivery`**, funds split atomically in one transaction:
- Restaurant receives food amount
- Driver receives delivery fee
- Treasury receives 0.02% protocol fee → `BiP5PJuUiXPYCFx98RMCGCnRhdUVrkxSke9C6y2ZohQ9`
- Customer deposit (2%) unlocked for claim

### Registry (`forkit_registry`)

On-chain identity for all participants:

| Instruction | Who | What |
|---|---|---|
| `register` | Anyone | Create a profile PDA (role: Restaurant/Driver/Customer) |
| `update_metadata` | Profile owner | Update name, location, zones |
| `rate_counterparty` | Post-order | Submit 1–5 star rating |
| `update_loyalty_points` | Loyalty program | Sync points balance |

### Loyalty (`forkit_loyalty`)

Points economy with tiered protocol-fee discounts:

| Tier | Points | Protocol fee discount |
|---|---|---|
| Bronze | 500 – 2,499 | 5% |
| Silver | 2,500 – 9,999 | 10% |
| Gold | 10,000 – 49,999 | 15% |
| Platinum | 50,000+ | 20% |

AI-routed orders earn **+50% bonus points**.

---

## Backend API

Base URL: `http://localhost:3001`

### Authentication
| Method | Path | Description |
|---|---|---|
| `POST` | `/api/auth/nonce` | Request a nonce to sign with your wallet |
| `POST` | `/api/auth/verify` | Submit signed nonce → JWT |

All other endpoints require `Authorization: Bearer <jwt>`.

### Customers
| Method | Path | Description |
|---|---|---|
| `POST` | `/api/customers/register` | Register customer profile |
| `GET` | `/api/customers/restaurants` | Browse restaurants with menus |
| `POST` | `/api/customers/orders` | Create order (generates CODE_A, CODE_B; returns on-chain params) |
| `GET` | `/api/customers/orders` | Order history |
| `GET` | `/api/customers/orders/:id` | Full order details (used by tracking page) |
| `GET` | `/api/customers/orders/:id/receipt` | Structured receipt with fee breakdown and settlement tx |
| `POST` | `/api/customers/orders/:id/cancel` | Cancel order (within 60s window) |
| `PATCH` | `/api/customers/orders/:id/settle` | Record on-chain settlement tx; emits `order:funds-released` |

### Restaurants
| Method | Path | Description |
|---|---|---|
| `POST` | `/api/restaurants/register` | Register restaurant |
| `PUT` | `/api/restaurants/profile` | Update profile |
| `POST` | `/api/restaurants/menu` | Add/update menu item with IPFS image |
| `GET` | `/api/restaurants/menu/:id` | Fetch menu |
| `GET` | `/api/restaurants/orders` | Incoming orders |

### Drivers
| Method | Path | Description |
|---|---|---|
| `POST` | `/api/drivers/register` | Register driver with delivery zones |
| `PUT` | `/api/drivers/zones` | Update zones and pricing |
| `GET` | `/api/drivers/available-orders` | Orders ready for pickup |
| `POST` | `/api/drivers/orders/:id/accept` | Claim a delivery |

### Contributions (Multi-sig orders)
| Method | Path | Description |
|---|---|---|
| `GET` | `/api/contributions/order/:orderId` | Funding progress |
| `GET` | `/api/contributions/share/:shareLink` | Public order details via share link |
| `POST` | `/api/contributions` | Record a contribution |
| `POST` | `/api/contributions/generate-link/:orderId` | Generate shareable funding link |

### Real-Time WebSocket Events (Socket.IO)

Connect to `ws://localhost:3001`. Authenticate with `{ token: jwt }`.

| Emit | Effect |
|---|---|
| `subscribe:order` + orderId | Join the order's room to receive events |
| `unsubscribe:order` + orderId | Leave the room |

| Event | Payload | Meaning |
|---|---|---|
| `order:created` | `{ orderId, timestamp }` | Order placed, escrow locked |
| `order:accepted` | `{ orderId, timestamp }` | Restaurant accepted |
| `order:preparing` | `{ orderId, timestamp }` | Cooking started |
| `order:ready` | `{ orderId, timestamp }` | Ready for pickup |
| `order:picked-up` | `{ orderId, timestamp, deliveryService }` | Driver collected (`deliveryService: 'human'\|'ai'`) |
| `order:delivered` | `{ orderId, timestamp }` | Delivered to customer |
| `order:settled` | `{ orderId, timestamp, txSignature }` | On-chain settlement confirmed |
| `order:funds-released` | `{ orderId, txSignature, totalReleased, restaurantReceived, driverReceived, depositRefunded, tokenSymbol }` | Full payment breakdown after settlement |
| `order:cancelled` | `{ orderId, timestamp }` | Order cancelled |
| `order:disputed` | `{ orderId, timestamp }` | Dispute opened |
| `order:refunded` | `{ orderId, timestamp }` | Funds returned |

---

## Frontend

See **[packages/frontend/README.md](packages/frontend/README.md)** for the full frontend guide including the order tracking page, currency selector, and receipt component.

---

## Data Model

```
Restaurant ←─── MenuItem
    │
    ├──── Order ────→ Customer
    │       │
    │       ├──────→ Driver
    │       │
    │       └──────→ Contribution[]
```

Orders reference on-chain escrow via `onChainOrderId`. Off-chain data (menus, metadata, zones) lives in PostgreSQL. Menu images are pinned to IPFS via Pinata.

---

## Development

```bash
# Run all packages in dev mode
npm run dev

# Build everything
npm run build

# Run contract tests
cd packages/contracts && anchor test

# Database operations
cd packages/backend
npx prisma studio       # GUI browser
npx prisma migrate dev  # apply migrations
```

---

## License

MIT
