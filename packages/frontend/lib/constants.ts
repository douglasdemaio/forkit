import { PublicKey } from '@solana/web3.js';

export const TREASURY_WALLET = new PublicKey('BiP5PJuUiXPYCFx98RMCGCnRhdUVrkxSke9C6y2ZohQ9');
export const FEE_BASIS_POINTS = 2; // 0.02%
export const DEPOSIT_BASIS_POINTS = 200; // 2%

export const SOLANA_RPC_URL = process.env.NEXT_PUBLIC_SOLANA_RPC_URL || 'https://api.devnet.solana.com';
export const SOLANA_NETWORK = (process.env.NEXT_PUBLIC_SOLANA_NETWORK || 'devnet') as 'devnet' | 'mainnet-beta';
export const API_URL = process.env.NEXT_PUBLIC_API_URL || 'http://localhost:3001';

export interface StablecoinToken {
  symbol: string;
  name: string;
  mint: string;
  decimals: number;
  /** Currency symbol shown in UI */
  currencySign: string;
  /** Tailwind color classes for the token badge */
  colorClass: string;
}

/** Stablecoins supported for payment (devnet addresses) */
export const SUPPORTED_TOKENS: StablecoinToken[] = [
  {
    symbol: 'USDC',
    name: 'USD Coin',
    // Devnet USDC (Circle)
    mint: '4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU',
    decimals: 6,
    currencySign: '$',
    colorClass: 'bg-blue-500',
  },
  {
    symbol: 'EURC',
    name: 'Euro Coin',
    // Devnet EURC – update to Circle's devnet deployment when available.
    // Mainnet: HzwqbKZw8HxMN6bF2yFZNrht3c2iXXzpKcFu7uBEDKtr
    mint: 'CXk2AMBfi3TwaEL2468s6zP8xq9NxTXjp9gjMgzeUynM',
    decimals: 6,
    currencySign: '€',
    colorClass: 'bg-yellow-500',
  },
];

/** Look up a token by mint address */
export function getTokenByMint(mint: string): StablecoinToken | undefined {
  return SUPPORTED_TOKENS.find((t) => t.mint === mint);
}

/** Known stablecoin mints for display (all networks) */
export const KNOWN_MINTS: Record<string, { symbol: string; name: string; decimals: number }> = {
  // Mainnet USDC
  EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v: { symbol: 'USDC', name: 'USD Coin', decimals: 6 },
  // Devnet USDC
  '4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU': { symbol: 'USDC', name: 'USD Coin', decimals: 6 },
  // Mainnet EURC
  HzwqbKZw8HxMN6bF2yFZNrht3c2iXXzpKcFu7uBEDKtr: { symbol: 'EURC', name: 'Euro Coin', decimals: 6 },
  // Devnet EURC placeholder
  CXk2AMBfi3TwaEL2468s6zP8xq9NxTXjp9gjMgzeUynM: { symbol: 'EURC', name: 'Euro Coin', decimals: 6 },
};

/** Solana Explorer base URL for the configured network */
export const EXPLORER_URL =
  SOLANA_NETWORK === 'mainnet-beta'
    ? 'https://explorer.solana.com'
    : 'https://explorer.solana.com?cluster=devnet';

export function explorerTxUrl(txSignature: string): string {
  return `https://explorer.solana.com/tx/${txSignature}${SOLANA_NETWORK !== 'mainnet-beta' ? '?cluster=devnet' : ''}`;
}
