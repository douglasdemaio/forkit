import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import { WalletConnectProvider, WalletButton } from '@/components/wallet-connect';
import Link from 'next/link';
import './globals.css';

const inter = Inter({ subsets: ['latin'] });

export const metadata: Metadata = {
  title: 'ForkIt — Decentralized Food Delivery',
  description: 'Trustless food delivery on Solana. 0.02% fees. No middlemen.',
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body className={inter.className}>
        <WalletConnectProvider>
          <nav className="fixed top-0 w-full z-50 bg-forkit-navy/95 backdrop-blur border-b border-gray-800">
            <div className="max-w-7xl mx-auto px-4 h-16 flex items-center justify-between">
              <Link href="/" className="flex items-center gap-2">
                <span className="text-2xl">🍴</span>
                <span className="text-xl font-bold text-white">ForkIt</span>
              </Link>
              <div className="flex items-center gap-6">
                <Link href="/customer/browse" className="text-gray-300 hover:text-white text-sm">
                  Order Food
                </Link>
                <Link href="/restaurant/dashboard" className="text-gray-300 hover:text-white text-sm">
                  Restaurants
                </Link>
                <Link href="/driver/dashboard" className="text-gray-300 hover:text-white text-sm">
                  Drive
                </Link>
                <WalletButton />
              </div>
            </div>
          </nav>
          <main className="pt-16">{children}</main>
        </WalletConnectProvider>
      </body>
    </html>
  );
}
