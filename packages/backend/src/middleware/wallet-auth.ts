import { Request, Response, NextFunction } from 'express';
import jwt from 'jsonwebtoken';
import nacl from 'tweetnacl';
import bs58 from 'bs58';
import { JWT_EXPIRY } from '../config/constants';

const JWT_SECRET = process.env.JWT_SECRET || 'forkit-dev-secret';

export interface AuthRequest extends Request {
  walletAddress?: string;
}

/**
 * Verify a Solana wallet signature for authentication.
 * Client signs a message like: "Sign in to ForkIt: <nonce>"
 * Then sends { walletAddress, signature, message } to get a JWT.
 */
export function verifyWalletSignature(
  walletAddress: string,
  signature: string,
  message: string
): boolean {
  try {
    const publicKey = bs58.decode(walletAddress);
    const sig = bs58.decode(signature);
    const msg = new TextEncoder().encode(message);
    return nacl.sign.detached.verify(msg, sig, publicKey);
  } catch {
    return false;
  }
}

export function generateToken(walletAddress: string): string {
  return jwt.sign({ walletAddress }, JWT_SECRET, { expiresIn: JWT_EXPIRY });
}

/**
 * Express middleware to verify JWT and attach walletAddress to request.
 */
export function authMiddleware(req: AuthRequest, res: Response, next: NextFunction): void {
  const authHeader = req.headers.authorization;
  if (!authHeader?.startsWith('Bearer ')) {
    res.status(401).json({ error: 'Missing authorization token' });
    return;
  }

  try {
    const token = authHeader.split(' ')[1];
    const decoded = jwt.verify(token, JWT_SECRET) as { walletAddress: string };
    req.walletAddress = decoded.walletAddress;
    next();
  } catch {
    res.status(401).json({ error: 'Invalid or expired token' });
  }
}
