'use client';

import { explorerTxUrl, getTokenByMint } from '@/lib/constants';
import type { OrderReceipt } from '@/lib/types';

interface PaymentReceiptProps {
  receipt: OrderReceipt;
}

function Row({ label, value, highlight }: { label: string; value: string; highlight?: boolean }) {
  return (
    <div className={`flex justify-between py-1.5 text-sm ${highlight ? 'font-semibold' : ''}`}>
      <span className={highlight ? 'text-white' : 'text-slate-400'}>{label}</span>
      <span className={highlight ? 'text-green-400' : 'text-slate-200'}>{value}</span>
    </div>
  );
}

function Divider() {
  return <div className="my-2 border-t border-slate-700" />;
}

function truncateTx(sig: string) {
  return `${sig.slice(0, 8)}…${sig.slice(-8)}`;
}

function formatAmount(amount: number, currencySign: string, decimals = 6) {
  // amount is in lamports/smallest unit – convert to human-readable
  const human = amount / Math.pow(10, decimals);
  return `${currencySign}${human.toFixed(2)}`;
}

function formatDate(iso: string) {
  return new Date(iso).toLocaleString([], {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}

export function PaymentReceipt({ receipt }: PaymentReceiptProps) {
  const token = getTokenByMint(receipt.tokenMint);
  const sign = receipt.currencySign || token?.currencySign || '$';
  const symbol = receipt.tokenSymbol || token?.symbol || 'USDC';
  const decimals = token?.decimals ?? 6;

  const handlePrint = () => window.print();

  return (
    <div className="rounded-xl border border-slate-700 bg-slate-800 p-5 print:border-0 print:bg-white print:text-black">
      {/* Header */}
      <div className="mb-4 flex items-start justify-between">
        <div>
          <p className="text-xs uppercase tracking-widest text-slate-500">ForkIt Receipt</p>
          <h2 className="mt-0.5 text-lg font-bold text-white print:text-black">
            {receipt.restaurantName}
          </h2>
          <p className="text-xs text-slate-500">{formatDate(receipt.createdAt)}</p>
        </div>
        <div className="text-right">
          <span
            className={`inline-block rounded-full px-2.5 py-0.5 text-xs font-medium ${
              receipt.status === 'Settled'
                ? 'bg-green-500/20 text-green-400'
                : receipt.status === 'Delivered'
                ? 'bg-blue-500/20 text-blue-400'
                : 'bg-slate-700 text-slate-300'
            }`}
          >
            {receipt.status}
          </span>
          <p className="mt-1 font-mono text-xs text-slate-600">
            #{receipt.onChainOrderId.toString().slice(-8)}
          </p>
        </div>
      </div>

      {/* Items */}
      <div className="space-y-1">
        {receipt.items.map((item, i) => (
          <div key={i} className="flex justify-between text-sm">
            <span className="text-slate-300">
              {item.quantity}× {item.name}
            </span>
            <span className="text-slate-200">
              {sign}
              {(item.price * item.quantity).toFixed(2)}
            </span>
          </div>
        ))}
      </div>

      <Divider />

      {/* Fee breakdown */}
      <Row label="Food subtotal" value={`${sign}${Number(receipt.foodTotal).toFixed(2)}`} />
      <Row label="Delivery fee" value={`${sign}${Number(receipt.deliveryFee).toFixed(2)}`} />
      <Row
        label={`Protocol fee (0.02%)`}
        value={`${sign}${Number(receipt.protocolFee).toFixed(2)}`}
      />
      <Row
        label="Security deposit (2% · refundable)"
        value={`+${sign}${Number(receipt.depositAmount).toFixed(2)}`}
      />

      <Divider />

      <Row
        label="Total charged"
        value={`${sign}${Number(receipt.totalCharged).toFixed(2)} ${symbol}`}
        highlight
      />

      {receipt.depositRefunded > 0 && (
        <Row
          label="Deposit refunded"
          value={`−${sign}${Number(receipt.depositRefunded).toFixed(2)}`}
        />
      )}

      {receipt.status === 'Settled' && (
        <Row
          label="Net paid"
          value={`${sign}${Number(receipt.netPaid).toFixed(2)} ${symbol}`}
          highlight
        />
      )}

      {/* Delivery type */}
      {receipt.deliveryService && (
        <>
          <Divider />
          <Row
            label="Delivery by"
            value={receipt.deliveryService === 'ai' ? '🤖 AI Delivery Service' : '🧑 Human Driver'}
          />
        </>
      )}

      {/* Settlement tx */}
      {receipt.settleTxSignature && (
        <>
          <Divider />
          <div className="flex items-center justify-between text-sm">
            <span className="text-slate-400">Funds released on-chain</span>
            <a
              href={explorerTxUrl(receipt.settleTxSignature)}
              target="_blank"
              rel="noopener noreferrer"
              className="font-mono text-xs text-green-500 hover:underline"
              title={receipt.settleTxSignature}
            >
              {truncateTx(receipt.settleTxSignature)} ↗
            </a>
          </div>
          <p className="mt-1 text-right text-xs text-slate-600">
            {receipt.settledAt && formatDate(receipt.settledAt)}
          </p>
        </>
      )}

      {/* Payment token */}
      <Divider />
      <div className="flex items-center justify-between text-xs text-slate-500">
        <span>Payment token</span>
        <span className="flex items-center gap-1.5">
          <span
            className={`inline-block h-3 w-3 rounded-full ${
              symbol === 'USDC' ? 'bg-blue-500' : 'bg-yellow-500'
            }`}
          />
          {symbol} · Solana
        </span>
      </div>

      {/* Print button */}
      <button
        onClick={handlePrint}
        className="mt-4 w-full rounded-lg border border-slate-600 py-2 text-sm text-slate-400 transition hover:border-slate-400 hover:text-white print:hidden"
      >
        Print / Save Receipt
      </button>
    </div>
  );
}
