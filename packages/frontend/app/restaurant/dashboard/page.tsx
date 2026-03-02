'use client';

import { useWallet } from '@solana/wallet-adapter-react';
import Link from 'next/link';

export default function RestaurantDashboard() {
  const { publicKey } = useWallet();

  if (!publicKey) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <div className="text-center">
          <h1 className="text-3xl font-bold mb-4">Restaurant Dashboard</h1>
          <p className="text-gray-400">Connect your wallet to get started.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-6xl mx-auto px-4 py-12">
      <h1 className="text-3xl font-bold mb-8">Restaurant Dashboard</h1>
      <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-6">
        <Link href="/restaurant/menu-editor" className="p-6 rounded-xl bg-forkit-slate border border-gray-700 hover:border-forkit-green transition-colors">
          <h3 className="text-lg font-semibold mb-2">📝 Menu Editor</h3>
          <p className="text-gray-400 text-sm">Add, edit, and manage your menu items.</p>
        </Link>
        <Link href="/restaurant/orders" className="p-6 rounded-xl bg-forkit-slate border border-gray-700 hover:border-forkit-green transition-colors">
          <h3 className="text-lg font-semibold mb-2">📦 Orders</h3>
          <p className="text-gray-400 text-sm">View incoming orders and manage fulfillment.</p>
        </Link>
        <Link href="/restaurant/profile" className="p-6 rounded-xl bg-forkit-slate border border-gray-700 hover:border-forkit-green transition-colors">
          <h3 className="text-lg font-semibold mb-2">⭐ Profile & Trust</h3>
          <p className="text-gray-400 text-sm">View your trust score, ratings, and earnings.</p>
        </Link>
      </div>
    </div>
  );
}
