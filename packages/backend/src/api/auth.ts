import { Router, Request, Response } from 'express';
import crypto from 'crypto';
import { verifyWalletSignature, generateToken } from '../middleware/wallet-auth';

const router = Router();

// Store nonces temporarily (in production, use Redis)
const nonces = new Map<string, { nonce: string; expires: number }>();

/**
 * POST /api/auth/nonce
 * Request a nonce for wallet signing.
 */
router.post('/nonce', (req: Request, res: Response) => {
  const { walletAddress } = req.body;
  if (!walletAddress) {
    res.status(400).json({ error: 'walletAddress required' });
    return;
  }

  const nonce = crypto.randomBytes(32).toString('hex');
  const message = `Sign in to ForkIt: ${nonce}`;
  nonces.set(walletAddress, { nonce, expires: Date.now() + 5 * 60 * 1000 }); // 5 min expiry

  res.json({ message, nonce });
});

/**
 * POST /api/auth/verify
 * Verify wallet signature and return JWT.
 */
router.post('/verify', (req: Request, res: Response) => {
  const { walletAddress, signature, message } = req.body;

  if (!walletAddress || !signature || !message) {
    res.status(400).json({ error: 'walletAddress, signature, and message required' });
    return;
  }

  // Verify nonce exists and hasn't expired
  const stored = nonces.get(walletAddress);
  if (!stored || stored.expires < Date.now()) {
    res.status(401).json({ error: 'Nonce expired or not found. Request a new one.' });
    return;
  }

  const expectedMessage = `Sign in to ForkIt: ${stored.nonce}`;
  if (message !== expectedMessage) {
    res.status(401).json({ error: 'Message does not match expected nonce' });
    return;
  }

  // Verify the signature
  if (!verifyWalletSignature(walletAddress, signature, message)) {
    res.status(401).json({ error: 'Invalid signature' });
    return;
  }

  // Clean up nonce
  nonces.delete(walletAddress);

  // Issue JWT
  const token = generateToken(walletAddress);
  res.json({ token, walletAddress });
});

export default router;
