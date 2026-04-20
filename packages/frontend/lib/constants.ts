import { PublicKey } from '@solana/web3.js';

export const TREASURY_WALLET = new PublicKey('BiP5PJuUiXPYCFx98RMCGCnRhdUVrkxSke9C6y2ZohQ9');
export const FEE_BASIS_POINTS = 2; // 0.02%
export const DEPOSIT_BASIS_POINTS = 200; // 2%

export const SOLANA_RPC_URL = process.env.NEXT_PUBLIC_SOLANA_RPC_URL || 'https://api.devnet.solana.com';
export const SOLANA_NETWORK = (process.env.NEXT_PUBLIC_SOLANA_NETWORK || 'devnet') as 'devnet' | 'mainnet-beta';
export const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';

// Stablecoin mints
export const KNOWN_MINTS: Record<string, { symbol: string; name: string; decimals: number }> = {
  // Mainnet USDC
  EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v: { symbol: 'USDC', name: 'USD Coin', decimals: 6 },
  // Mainnet EURC
  HzwqbKZw8HxMN6bF2yFZNrht3c2iXXzpKcFu7uBEDKtr: { symbol: 'EURC', name: 'Euro Coin', decimals: 6 },
};

export const ESCROW_PROGRAM_ID = process.env.NEXT_PUBLIC_ESCROW_PROGRAM_ID || 'FNZXjjq2oceq15jVsnHT8gYJQUZ9NLCXCpYak2pXsqGB';
export const REGISTRY_PROGRAM_ID = process.env.NEXT_PUBLIC_REGISTRY_PROGRAM_ID || '2riHMdVB6eFgeQjqvnqq2Mrpqea7hrMv5ZNRh7gZgB9S';
export const LOYALTY_PROGRAM_ID = process.env.NEXT_PUBLIC_LOYALTY_PROGRAM_ID || '6DaFmi7haz2Ci9sXaHRviz3biwbmTwipvwc9L9cdeugR';

export const DEVNET_USDC_MINT = '4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU';
export const DEVNET_EURC_MINT = 'CXk2AMBfi3TwaEL2468s6zP8xq9NxTXjp9gjMgzeUynM';
