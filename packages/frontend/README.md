# ForkIt Frontend

The ForkIt web app — built with **Next.js 14**, **Tailwind CSS**, and **Solana Wallet Adapter**. Connects customers, restaurants, and drivers to the ForkIt decentralised food delivery protocol.

## Pages

| Route | Description |
|---|---|
| `/` | Landing page — role selector, feature overview, wallet connect |
| `/restaurant/dashboard` | Restaurant dashboard — manage menu, view & accept orders |
| `/order/[id]` | **Order tracking page** — live status timeline, payment summary, receipt |

---

## Order Tracking (`/order/[id]`)

The tracking page gives customers a real-time view of their order from placement to on-chain settlement.

### Status Timeline

The `OrderTracker` component shows a five-stage animated vertical timeline:

| # | Stage | On-chain status |
|---|---|---|
| 1 | Order Placed | `Created` |
| 2 | Cooking | `Preparing` |
| 3 | Ready for Pickup | `ReadyForPickup` |
| 4 | On the Way | `PickedUp` |
| 5 | Delivered & Funds Released | `Delivered` / `Settled` |

- Completed stages turn **green** with a checkmark
- The active stage **pulses** with a spinner
- Each stage shows the timestamp it was reached
- Stage 4 shows a **🤖 AI Routing** or **🧑 Human Driver** badge depending on `deliveryService`
- Stage 5 links directly to the settlement transaction on **Solana Explorer**

### Funds Release Banner

When `order:funds-released` is received via WebSocket the page shows:

```
✓ Funds Released On-Chain
$12.45 USDC released · deposit of $0.30 returned to your wallet.
[View transaction ↗]
```

The escrow indicator switches from **🔒 held in escrow** to **🔓 Escrow settled**.

---

## USDC / EURC Payments

Customers choose their stablecoin at checkout via the **CurrencySelector** component.

| Token | Devnet mint | Mainnet mint |
|---|---|---|
| **USDC** | `4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU` | `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v` |
| **EURC** | `CXk2AMBfi3TwaEL2468s6zP8xq9NxTXjp9gjMgzeUynM` | `HzwqbKZw8HxMN6bF2yFZNrht3c2iXXzpKcFu7uBEDKtr` |

Mints are defined in `lib/constants.ts` → `SUPPORTED_TOKENS`. To add another token, append an entry to that array and whitelist the mint on-chain via `add_accepted_mint`.

---

## In-App Receipt

After settlement the `PaymentReceipt` component shows:

- Itemised food lines (quantity × name × price)
- Delivery fee
- Protocol fee (0.02% → treasury `BiP5PJuUiXPYCFx98RMCGCnRhdUVrkxSke9C6y2ZohQ9`)
- Security deposit + refund status
- Net amount paid
- Token symbol (USDC / EURC)
- On-chain settlement transaction link
- **Print / Save** button

---

## Components

| Component | File | Description |
|---|---|---|
| `WalletConnect` | `components/wallet-connect.tsx` | Phantom / Solflare / Backpack modal |
| `CurrencySelector` | `components/currency-selector.tsx` | USDC / EURC card picker |
| `OrderTracker` | `components/order-tracker.tsx` | Animated 5-stage status timeline |
| `PaymentReceipt` | `components/payment-receipt.tsx` | Itemised receipt with tx link and print |
| `PriceBreakdown` | `components/price-breakdown.tsx` | Pre-order fee calculator |
| `CodeInput` | `components/code-input.tsx` | 6-character alphanumeric delivery code input |
| `TrustBadge` | `components/trust-badge.tsx` | On-chain reputation score display |

---

## Hooks

| Hook | File | Description |
|---|---|---|
| `useWalletAuth` | `hooks/useWalletAuth.ts` | Nonce-based wallet signature → JWT |
| `useOrderTracking` | `hooks/useOrderTracking.ts` | Fetches order + receipt; subscribes to live WebSocket events; maintains `statusHistory[]` |
| `useOrderStatus` | `hooks/useOrderStatus.ts` | Lightweight Socket.IO subscriber (status only) |
| `useEscrow` | `hooks/useEscrow.ts` | PDA derivation helpers for on-chain escrow accounts |

### `useOrderTracking`

```ts
const {
  order,          // Order | null
  currentStatus,  // OrderStatus | null
  statusHistory,  // StatusEvent[]  — timestamped list of every transition
  receipt,        // OrderReceipt | null
  fundsReleased,  // FundsReleasedPayload | null — set when order:funds-released fires
  isLoading,
  error,
} = useOrderTracking(orderId, authToken);
```

`StatusEvent`:

```ts
interface StatusEvent {
  status: OrderStatus;
  timestamp: string;        // ISO 8601
  txSignature?: string;     // on-chain tx (set for Settled stage)
  deliveryService?: 'human' | 'ai';
  note?: string;            // human-readable description
}
```

---

## Types (`lib/types.ts`)

```ts
type OrderStatus =
  | 'Created' | 'Preparing' | 'ReadyForPickup' | 'PickedUp'
  | 'Delivered' | 'Settled' | 'Disputed' | 'Cancelled' | 'Refunded';

type DeliveryService = 'human' | 'ai';

interface OrderReceipt {
  orderId, onChainOrderId, restaurantName
  items: CartItem[]
  tokenMint, tokenSymbol, currencySign
  foodTotal, deliveryFee, protocolFee, depositAmount
  depositRefunded, totalCharged, netPaid
  status, createdAt, settledAt?
  settleTxSignature?        // on-chain settlement tx
  deliveryService?
}

interface FundsReleasedPayload {
  orderId, txSignature
  totalReleased, restaurantReceived, driverReceived, depositRefunded
  tokenSymbol
}
```

---

## Constants (`lib/constants.ts`)

| Export | Value | Purpose |
|---|---|---|
| `SUPPORTED_TOKENS` | Array of `StablecoinToken` | USDC + EURC with mint addresses and currency signs |
| `TREASURY_WALLET` | `BiP5PJuUiXPYCFx98RMCGCnRhdUVrkxSke9C6y2ZohQ9` | Receives the 0.02% protocol fee |
| `FEE_BASIS_POINTS` | `2` | Protocol fee rate (0.02%) |
| `DEPOSIT_BASIS_POINTS` | `200` | Customer deposit rate (2%, refundable) |
| `API_URL` | `http://localhost:3001` | Backend base URL |
| `SOLANA_RPC_URL` | devnet by default | Solana RPC endpoint |
| `explorerTxUrl(sig)` | function | Builds Solana Explorer URL for a tx signature |
| `getTokenByMint(mint)` | function | Look up `StablecoinToken` by mint address |

---

## Environment Variables

Create `packages/frontend/.env.local`:

```env
NEXT_PUBLIC_SOLANA_RPC_URL=https://api.devnet.solana.com
NEXT_PUBLIC_SOLANA_NETWORK=devnet
NEXT_PUBLIC_API_URL=http://localhost:3001
```

---

## Running

```bash
# From repo root (recommended — starts backend too)
npm run dev

# Frontend only
cd packages/frontend
npm run dev      # http://localhost:3000
npm run build
npm run start
```

---

## Wallet Support

Supported wallets via `@solana/wallet-adapter-wallets`:

- [Phantom](https://phantom.app)
- [Solflare](https://solflare.com)
- [Backpack](https://backpack.app)

Authentication flow: wallet signs a nonce message → backend verifies signature via TweetNaCl → returns JWT stored in `localStorage` as `forkit_token`.
