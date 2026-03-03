export const TREASURY_WALLET = 'BiP5PJuUiXPYCFx98RMCGCnRhdUVrkxSke9C6y2ZohQ9';
export const FEE_BASIS_POINTS = 2; // 0.02%
export const CANCEL_WINDOW_SECONDS = 60;
export const PREP_TIMEOUT_SECONDS = 2700;
export const PICKUP_TIMEOUT_SECONDS = 1800;
export const DELIVERY_TIMEOUT_SECONDS = 7200;
export const DEPOSIT_BASIS_POINTS = 200; // 2%
export const JWT_EXPIRY = '24h';
export const MAX_IMAGE_SIZE = 5 * 1024 * 1024; // 5MB
export const SUPPORTED_IMAGE_TYPES = ['image/jpeg', 'image/png', 'image/webp'];

// Solana
export const SOLANA_RPC_URL = process.env.SOLANA_RPC_URL || 'https://api.devnet.solana.com';
export const ESCROW_PROGRAM_ID = process.env.ESCROW_PROGRAM_ID || '';
export const REGISTRY_PROGRAM_ID = process.env.REGISTRY_PROGRAM_ID || '';
