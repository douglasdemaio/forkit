import { PublicKey } from '@solana/web3.js';

export const TREASURY_WALLET = new PublicKey('BiP5PJuUiXPYCFx98RMCGCnRhdUVrkxSke9C6y2ZohQ9');
export const FEE_BASIS_POINTS = 2; // 0.02%
export const DEPOSIT_MULTIPLIER = 2;

export const SOLANA_RPC_URL = process.env.NEXT_PUBLIC_SOLANA_RPC_URL || 'https://api.devnet.solana.com';
export const SOLANA_NETWORK = (process.env.NEXT_PUBLIC_SOLANA_NETWORK || 'devnet') as 'devnet' | 'mainnet-beta';
export const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';

// Known stablecoin mints (devnet)
export const KNOWN_MINTS: Record<string, { symbol: string; name: string; decimals: number }> = {
  // Mainnet USDC
  EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v: { symbol: 'USDC', name: 'USD Coin', decimals: 6 },
  // Mainnet EURC
  HzwqbKZw8HxMN6bF2yFZNrht3c2iXXzpKcFu7uBEDKtr: { symbol: 'EURC', name: 'Euro Coin', decimals: 6 },
};
