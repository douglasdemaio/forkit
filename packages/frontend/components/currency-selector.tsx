'use client';

import { SUPPORTED_TOKENS, StablecoinToken } from '@/lib/constants';

interface CurrencySelectorProps {
  selected: StablecoinToken | null;
  onSelect: (token: StablecoinToken) => void;
  className?: string;
}

export function CurrencySelector({ selected, onSelect, className = '' }: CurrencySelectorProps) {
  return (
    <div className={`space-y-2 ${className}`}>
      <p className="text-sm font-medium text-slate-300">Pay with</p>
      <div className="grid grid-cols-2 gap-3">
        {SUPPORTED_TOKENS.map((token) => {
          const isSelected = selected?.mint === token.mint;
          return (
            <button
              key={token.mint}
              type="button"
              onClick={() => onSelect(token)}
              className={`relative flex items-center gap-3 rounded-xl border px-4 py-3 text-left transition-all duration-150 focus:outline-none focus:ring-2 focus:ring-green-500 ${
                isSelected
                  ? 'border-green-500 bg-green-500/10 shadow-[0_0_0_1px_rgba(16,185,129,0.4)]'
                  : 'border-slate-700 bg-slate-800 hover:border-slate-500'
              }`}
              aria-pressed={isSelected}
            >
              {/* Token icon */}
              <span
                className={`flex h-9 w-9 shrink-0 items-center justify-center rounded-full text-base font-bold text-white ${token.colorClass}`}
              >
                {token.currencySign}
              </span>

              <div className="min-w-0">
                <p className="font-semibold text-white">{token.symbol}</p>
                <p className="truncate text-xs text-slate-400">{token.name}</p>
              </div>

              {isSelected && (
                <span className="absolute right-3 top-3 flex h-4 w-4 items-center justify-center rounded-full bg-green-500">
                  <svg className="h-2.5 w-2.5 text-white" fill="none" viewBox="0 0 10 10">
                    <path
                      d="M1.5 5L3.8 7.5L8.5 2.5"
                      stroke="currentColor"
                      strokeWidth="1.5"
                      strokeLinecap="round"
                      strokeLinejoin="round"
                    />
                  </svg>
                </span>
              )}
            </button>
          );
        })}
      </div>

      {selected && (
        <p className="text-xs text-slate-500">
          Funds held in escrow as {selected.symbol} · released automatically on delivery
        </p>
      )}
    </div>
  );
}
