'use client';

import { useState, useEffect, useCallback } from 'react';
import { useWallet, useConnection } from '@solana/wallet-adapter-react';
import { PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import { getAssociatedTokenAddress, TOKEN_PROGRAM_ID } from '@solana/spl-token';

const ESCROW_PROGRAM_ID = new PublicKey(
  process.env.NEXT_PUBLIC_ESCROW_PROGRAM_ID || 'FNZXjjq2oceq15jVsnHT8gYJQUZ9NLCXCpYak2pXsqGB'
);
const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';

type Resolution = 'RefundCustomer' | 'PayRestaurantAndDriver' | 'Split';

const RESOLUTION_BYTE: Record<Resolution, number> = {
  RefundCustomer: 0,
  PayRestaurantAndDriver: 1,
  Split: 2,
};

interface DisputedOrder {
  id: string;
  onChainOrderId: string;
  tokenMint: string;
  foodTotal: number;
  deliveryFee: number;
  depositAmount: number;
  escrowTarget: number;
  status: string;
  restaurant: { name: string; walletAddress: string };
  contributions: { walletAddress: string; amount: number }[];
  // stored in metadata or as top-level fields depending on schema
  customerWallet?: string;
  driverWallet?: string;
}

async function getResolveDisputeDiscriminator(): Promise<Uint8Array> {
  const hash = await crypto.subtle.digest(
    'SHA-256',
    new TextEncoder().encode('global:resolve_dispute')
  );
  return new Uint8Array(hash).slice(0, 8);
}

function orderIdToLE(orderId: string): Buffer {
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64LE(BigInt(orderId));
  return buf;
}

export default function AdminDisputesPage() {
  const { publicKey, sendTransaction } = useWallet();
  const { connection } = useConnection();
  const [orders, setOrders] = useState<DisputedOrder[]>([]);
  const [loading, setLoading] = useState(false);
  const [resolving, setResolving] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const authHeader = () => {
    const token = typeof window !== 'undefined' ? localStorage.getItem('forkit_token') : null;
    return token ? { Authorization: `Bearer ${token}` } : {};
  };

  const fetchDisputes = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const res = await fetch(`${API_URL}/api/admin/disputes`, {
        headers: { ...authHeader(), 'Content-Type': 'application/json' },
      });
      if (!res.ok) throw new Error(res.status === 403 ? 'Admin access required' : 'Failed to load disputes');
      setOrders(await res.json());
    } catch (e: any) {
      setError(e.message);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    if (publicKey) fetchDisputes();
  }, [publicKey, fetchDisputes]);

  const resolve = async (order: DisputedOrder, resolution: Resolution) => {
    if (!publicKey) return;
    setResolving(order.id);
    setError(null);
    try {
      const discriminator = await getResolveDisputeDiscriminator();
      const orderIdBytes = orderIdToLE(order.onChainOrderId);

      const [orderPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('order'), orderIdBytes],
        ESCROW_PROGRAM_ID
      );
      const [protocolConfigPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('protocol_config')],
        ESCROW_PROGRAM_ID
      );
      const [escrowVaultPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from('escrow_vault'), orderIdBytes],
        ESCROW_PROGRAM_ID
      );

      const tokenMint = new PublicKey(order.tokenMint);
      const restaurantPubkey = new PublicKey(order.restaurant.walletAddress);
      const driverPubkey = order.driverWallet ? new PublicKey(order.driverWallet) : publicKey;

      const restaurantTokenAccount = await getAssociatedTokenAddress(tokenMint, restaurantPubkey);
      const driverTokenAccount = await getAssociatedTokenAddress(tokenMint, driverPubkey);

      const data = Buffer.concat([
        Buffer.from(discriminator),
        Buffer.from([RESOLUTION_BYTE[resolution]]),
      ]);

      const ix = new TransactionInstruction({
        programId: ESCROW_PROGRAM_ID,
        keys: [
          { pubkey: orderPDA, isSigner: false, isWritable: true },
          { pubkey: protocolConfigPDA, isSigner: false, isWritable: false },
          { pubkey: escrowVaultPDA, isSigner: false, isWritable: true },
          { pubkey: restaurantTokenAccount, isSigner: false, isWritable: true },
          { pubkey: driverTokenAccount, isSigner: false, isWritable: true },
          { pubkey: publicKey, isSigner: true, isWritable: false },
          { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
        ],
        data,
      });

      const tx = new Transaction().add(ix);
      const txSignature = await sendTransaction(tx, connection);
      await connection.confirmTransaction(txSignature, 'confirmed');

      const patchRes = await fetch(`${API_URL}/api/admin/disputes/${order.id}/resolve`, {
        method: 'PATCH',
        headers: { ...authHeader(), 'Content-Type': 'application/json' },
        body: JSON.stringify({ resolution, txSignature }),
      });
      if (!patchRes.ok) throw new Error('On-chain tx succeeded but failed to update database');

      setOrders((prev) => prev.filter((o) => o.id !== order.id));
    } catch (e: any) {
      setError(e.message);
    } finally {
      setResolving(null);
    }
  };

  if (!publicKey) {
    return (
      <div className="min-h-screen flex items-center justify-center bg-forkit-navy">
        <div className="text-center">
          <h1 className="text-3xl font-bold mb-4 text-white">Admin — Dispute Resolution</h1>
          <p className="text-gray-400">Connect your admin wallet to continue.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-5xl mx-auto px-4 py-12">
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-3xl font-bold text-white">Dispute Resolution</h1>
          <p className="text-gray-400 text-sm mt-1">
            {orders.length} open dispute{orders.length !== 1 ? 's' : ''}
          </p>
        </div>
        <button
          onClick={fetchDisputes}
          disabled={loading}
          className="px-4 py-2 rounded-lg bg-forkit-slate border border-gray-700 text-sm text-gray-300 hover:border-forkit-green transition-colors disabled:opacity-50"
        >
          {loading ? 'Loading…' : 'Refresh'}
        </button>
      </div>

      {error && (
        <div className="mb-6 p-4 rounded-xl bg-red-900/30 border border-forkit-red text-forkit-red text-sm">
          {error}
        </div>
      )}

      {!loading && orders.length === 0 && !error && (
        <div className="p-12 rounded-xl bg-forkit-slate border border-gray-700 text-center text-gray-400">
          No open disputes.
        </div>
      )}

      <div className="space-y-6">
        {orders.map((order) => (
          <div key={order.id} className="rounded-xl bg-forkit-slate border border-forkit-amber p-6">
            <div className="flex items-start justify-between mb-4">
              <div>
                <span className="inline-block px-2 py-0.5 rounded text-xs font-mono bg-forkit-amber/20 text-forkit-amber mb-2">
                  DISPUTED
                </span>
                <h2 className="text-lg font-semibold text-white">{order.restaurant.name}</h2>
                <p className="text-gray-400 text-xs font-mono mt-1">
                  On-chain ID: {order.onChainOrderId}
                </p>
              </div>
              <div className="text-right text-sm">
                <p className="text-gray-400">
                  Food: <span className="text-white">{Number(order.foodTotal).toFixed(2)} USDC</span>
                </p>
                <p className="text-gray-400">
                  Delivery: <span className="text-white">{Number(order.deliveryFee).toFixed(2)} USDC</span>
                </p>
                <p className="text-gray-400">
                  Deposit: <span className="text-white">{Number(order.depositAmount).toFixed(2)} USDC</span>
                </p>
              </div>
            </div>

            <div className="grid grid-cols-2 gap-4 mb-4 text-xs">
              <div>
                <p className="text-gray-500 mb-1">Restaurant wallet</p>
                <p className="font-mono text-gray-300 truncate">{order.restaurant.walletAddress}</p>
              </div>
              {order.driverWallet && (
                <div>
                  <p className="text-gray-500 mb-1">Driver wallet</p>
                  <p className="font-mono text-gray-300 truncate">{order.driverWallet}</p>
                </div>
              )}
            </div>

            {order.contributions.length > 0 && (
              <div className="mb-4">
                <p className="text-gray-500 text-xs mb-2">
                  Contributors ({order.contributions.length})
                </p>
                <div className="space-y-1">
                  {order.contributions.map((c, i) => (
                    <div key={i} className="flex justify-between text-xs">
                      <span className="font-mono text-gray-400 truncate mr-4">{c.walletAddress}</span>
                      <span className="text-white whitespace-nowrap">{Number(c.amount).toFixed(2)} USDC</span>
                    </div>
                  ))}
                </div>
              </div>
            )}

            <div className="border-t border-gray-700 pt-4">
              <p className="text-gray-500 text-xs mb-3">Resolve dispute — this signs an on-chain transaction:</p>
              <div className="flex flex-wrap gap-3">
                {(['RefundCustomer', 'PayRestaurantAndDriver', 'Split'] as Resolution[]).map((r) => (
                  <button
                    key={r}
                    onClick={() => resolve(order, r)}
                    disabled={resolving === order.id}
                    className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors disabled:opacity-50 ${
                      r === 'RefundCustomer'
                        ? 'bg-forkit-red/20 border border-forkit-red text-forkit-red hover:bg-forkit-red/30'
                        : r === 'PayRestaurantAndDriver'
                        ? 'bg-forkit-green/20 border border-forkit-green text-forkit-green hover:bg-forkit-green/30'
                        : 'bg-forkit-amber/20 border border-forkit-amber text-forkit-amber hover:bg-forkit-amber/30'
                    }`}
                  >
                    {resolving === order.id ? 'Sending…' : r === 'RefundCustomer' ? 'Refund Customer' : r === 'PayRestaurantAndDriver' ? 'Pay Restaurant & Driver' : 'Split 50/50'}
                  </button>
                ))}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
