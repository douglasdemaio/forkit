'use client';

import { useParams } from 'next/navigation';
import { useOrderTracking } from '@/hooks/useOrderTracking';
import { OrderTracker } from '@/components/order-tracker';
import { PaymentReceipt } from '@/components/payment-receipt';
import { getTokenByMint } from '@/lib/constants';

export default function OrderTrackingPage() {
  const params = useParams();
  const orderId = typeof params.id === 'string' ? params.id : null;

  // Auth token would come from your wallet auth flow (Zustand / localStorage)
  const authToken =
    typeof window !== 'undefined' ? localStorage.getItem('forkit_token') ?? undefined : undefined;

  const { order, currentStatus, statusHistory, receipt, fundsReleased, isLoading, error } =
    useOrderTracking(orderId, authToken ?? undefined);

  if (isLoading) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-slate-950">
        <div className="flex flex-col items-center gap-3">
          <div className="h-10 w-10 animate-spin rounded-full border-2 border-slate-700 border-t-green-500" />
          <p className="text-sm text-slate-400">Loading your order…</p>
        </div>
      </main>
    );
  }

  if (error || !order || !currentStatus) {
    return (
      <main className="flex min-h-screen items-center justify-center bg-slate-950 px-4">
        <div className="text-center">
          <p className="text-2xl font-bold text-white">Order not found</p>
          <p className="mt-2 text-sm text-slate-400">{error ?? 'We could not locate this order.'}</p>
          <a href="/" className="mt-6 inline-block text-sm text-green-500 hover:underline">
            ← Back to home
          </a>
        </div>
      </main>
    );
  }

  const token = getTokenByMint(order.tokenMint);
  const sign = token?.currencySign ?? '$';
  const symbol = token?.symbol ?? 'USDC';
  const isSettled = currentStatus === 'Settled' || currentStatus === 'Delivered';

  return (
    <main className="min-h-screen bg-slate-950 px-4 py-10 text-white">
      <div className="mx-auto max-w-lg space-y-6">

        {/* Header */}
        <div>
          <a href="/" className="text-xs text-slate-500 hover:text-slate-300">
            ← Back
          </a>
          <h1 className="mt-3 text-2xl font-bold">Track Your Order</h1>
          <p className="mt-1 text-sm text-slate-400">
            {order.restaurant?.name ?? 'Restaurant'} ·{' '}
            <span className="font-mono text-xs text-slate-600">
              #{order.onChainOrderId.toString().slice(-8)}
            </span>
          </p>
        </div>

        {/* Funds-released banner */}
        {fundsReleased && (
          <div className="rounded-xl border border-green-500/30 bg-green-500/10 p-4">
            <p className="font-semibold text-green-400">✓ Funds Released On-Chain</p>
            <p className="mt-1 text-sm text-slate-300">
              {sign}
              {(fundsReleased.totalReleased / 1e6).toFixed(2)} {symbol} released · deposit of{' '}
              {sign}
              {(fundsReleased.depositRefunded / 1e6).toFixed(2)} returned to your wallet.
            </p>
            {fundsReleased.txSignature && (
              <a
                href={`https://explorer.solana.com/tx/${fundsReleased.txSignature}?cluster=devnet`}
                target="_blank"
                rel="noopener noreferrer"
                className="mt-2 inline-block text-xs text-green-500 hover:underline"
              >
                View transaction ↗
              </a>
            )}
          </div>
        )}

        {/* Order timeline */}
        <section className="rounded-xl border border-slate-800 bg-slate-900 p-5">
          <h2 className="mb-5 text-sm font-semibold uppercase tracking-wider text-slate-400">
            Order Status
          </h2>
          <OrderTracker
            currentStatus={currentStatus}
            statusHistory={statusHistory}
          />
        </section>

        {/* Payment summary card */}
        <section className="rounded-xl border border-slate-800 bg-slate-900 p-5">
          <h2 className="mb-3 text-sm font-semibold uppercase tracking-wider text-slate-400">
            Payment
          </h2>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-slate-400">Food subtotal</span>
              <span className="text-slate-200">
                {sign}{Number(order.foodTotal).toFixed(2)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-slate-400">Delivery fee</span>
              <span className="text-slate-200">
                {sign}{Number(order.deliveryFee).toFixed(2)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-slate-400">Protocol fee</span>
              <span className="text-slate-200">
                {sign}{Number(order.protocolFee).toFixed(2)}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-slate-400">Security deposit</span>
              <span className="text-slate-200">
                +{sign}{Number(order.depositAmount).toFixed(2)}
                {isSettled && (
                  <span className="ml-1 text-xs text-green-500">(returned)</span>
                )}
              </span>
            </div>
            <div className="mt-2 flex justify-between border-t border-slate-700 pt-2 font-semibold">
              <span className="text-white">Total</span>
              <span className="text-green-400">
                {sign}
                {(
                  Number(order.foodTotal) +
                  Number(order.deliveryFee) +
                  Number(order.protocolFee) +
                  Number(order.depositAmount)
                ).toFixed(2)}{' '}
                {symbol}
              </span>
            </div>
          </div>

          {/* Escrow status */}
          <div
            className={`mt-3 flex items-center gap-2 rounded-lg px-3 py-2 text-xs ${
              isSettled
                ? 'bg-green-500/10 text-green-400'
                : 'bg-amber-500/10 text-amber-400'
            }`}
          >
            <span>{isSettled ? '🔓' : '🔒'}</span>
            <span>
              {isSettled
                ? `Escrow settled · funds released on Solana`
                : `${sign}${(
                    Number(order.foodTotal) +
                    Number(order.deliveryFee) +
                    Number(order.protocolFee)
                  ).toFixed(2)} ${symbol} held in escrow`}
            </span>
          </div>
        </section>

        {/* Receipt (shown when settled or delivered) */}
        {receipt && isSettled && (
          <section>
            <h2 className="mb-3 text-sm font-semibold uppercase tracking-wider text-slate-400">
              Receipt
            </h2>
            <PaymentReceipt receipt={receipt} />
          </section>
        )}

        {/* Order items */}
        <section className="rounded-xl border border-slate-800 bg-slate-900 p-5">
          <h2 className="mb-3 text-sm font-semibold uppercase tracking-wider text-slate-400">
            Items
          </h2>
          <ul className="divide-y divide-slate-800">
            {order.items.map((item, i) => (
              <li key={i} className="flex justify-between py-2 text-sm">
                <span className="text-slate-300">
                  {item.quantity}× {item.name}
                </span>
                <span className="text-slate-200">
                  {sign}{(item.price * item.quantity).toFixed(2)}
                </span>
              </li>
            ))}
          </ul>
        </section>

      </div>
    </main>
  );
}
