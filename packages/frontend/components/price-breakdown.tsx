'use client';

import { FC } from 'react';
import { FEE_BASIS_POINTS, DEPOSIT_BASIS_POINTS, KNOWN_MINTS } from '@/lib/constants';

interface PriceBreakdownProps {
  foodTotal: number;
  deliveryFee: number;
  tokenMint: string;
  decimals?: number;
}

export const PriceBreakdown: FC<PriceBreakdownProps> = ({
  foodTotal,
  deliveryFee,
  tokenMint,
  decimals = 6,
}) => {
  const total = foodTotal + deliveryFee;
  const protocolFee = Math.ceil((total * FEE_BASIS_POINTS) / 10000);
  const deposit = Math.ceil((total * DEPOSIT_BASIS_POINTS) / 10000);
  const totalCharged = total + deposit;

  const mintInfo = KNOWN_MINTS[tokenMint];
  const symbol = mintInfo?.symbol || 'TOKEN';
  const fmt = (amount: number) => (amount / 10 ** decimals).toFixed(2);

  return (
    <div className="bg-forkit-slate rounded-xl p-6 space-y-3">
      <h3 className="text-lg font-semibold text-white mb-4">Order Summary</h3>

      <div className="flex justify-between text-gray-300">
        <span>Food subtotal</span>
        <span>{fmt(foodTotal)} {symbol}</span>
      </div>
      <div className="flex justify-between text-gray-300">
        <span>Delivery fee</span>
        <span>{fmt(deliveryFee)} {symbol}</span>
      </div>
      <div className="flex justify-between text-gray-400 text-sm">
        <span>Protocol fee (0.02%)</span>
        <span>{fmt(protocolFee)} {symbol}</span>
      </div>

      <div className="border-t border-gray-700 pt-3">
        <div className="flex justify-between text-gray-300">
          <span>Security deposit (2%)</span>
          <span>{fmt(deposit)} {symbol}</span>
        </div>
        <p className="text-xs text-gray-500 mt-1">
          Returned in full upon delivery confirmation
        </p>
      </div>

      <div className="border-t border-gray-700 pt-3">
        <div className="flex justify-between text-white font-bold text-lg">
          <span>Total charged</span>
          <span>{fmt(totalCharged)} {symbol}</span>
        </div>
        <p className="text-xs text-forkit-green mt-1">
          You pay {fmt(total)} {symbol} + {fmt(deposit)} {symbol} refundable deposit
        </p>
      </div>
    </div>
  );
};
