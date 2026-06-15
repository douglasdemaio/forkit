import { Request, Response, NextFunction } from 'express';
import { PublicKey } from '@solana/web3.js';
import nacl from 'tweetnacl';
import bs58 from 'bs58';
import jwt from 'jsonwebtoken';

const JWT_SECRET = process.env.JWT_SECRET || 'forkit-dev-secret-change-in-production';

export interface AuthRequest extends Request {
  walletAddress?: string;
}

export function verifyWalletSignature(
  walletAddress: string,
  signature: string,
  message: string
): boolean {
  try {
    const publicKey = new PublicKey(walletAddress);
    const messageBytes = new TextEncoder().encode(message);
    const signatureBytes = bs58.decode(signature);
    return nacl.sign.detached.verify(
      messageBytes,
      signatureBytes,
      publicKey.toBytes()
    );
  } catch {
    return false;
  }
}

export function generateToken(walletAddress: string): string {
  return jwt.sign({ walletAddress }, JWT_SECRET, { expiresIn: '24h' });
}

export function authMiddleware(
  req: AuthRequest,
  res: Response,
  next: NextFunction
): void {
  const header = req.headers.authorization;
  if (!header || !header.startsWith('Bearer ')) {
    res.status(401).json({ error: 'Missing or invalid authorization header' });
    return;
  }

  try {
    const token = header.slice(7);
    const payload = jwt.verify(token, JWT_SECRET) as { walletAddress: string };
    req.walletAddress = payload.walletAddress;
    next();
  } catch {
    res.status(401).json({ error: 'Invalid or expired token' });
  }
}
