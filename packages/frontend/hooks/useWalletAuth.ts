'use client';

import { useCallback, useState } from 'react';
import { useWallet } from '@solana/wallet-adapter-react';
import { API_URL } from '@/lib/constants';

export function useWalletAuth() {
  const { publicKey, signMessage } = useWallet();
  const [token, setToken] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const authenticate = useCallback(async () => {
    if (!publicKey || !signMessage) {
      throw new Error('Wallet not connected');
    }

    setLoading(true);
    try {
      // Request nonce
      const nonceRes = await fetch(`${API_URL}/api/auth/nonce`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ walletAddress: publicKey.toBase58() }),
      });
      const { message } = await nonceRes.json();

      // Sign message
      const encoded = new TextEncoder().encode(message);
      const signature = await signMessage(encoded);

      // Verify and get JWT
      const verifyRes = await fetch(`${API_URL}/api/auth/verify`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          walletAddress: publicKey.toBase58(),
          signature: Buffer.from(signature).toString('base64'),
          message,
        }),
      });

      const data = await verifyRes.json();
      if (data.token) {
        setToken(data.token);
        return data.token;
      }
      throw new Error(data.error || 'Authentication failed');
    } finally {
      setLoading(false);
    }
  }, [publicKey, signMessage]);

  return { token, loading, authenticate, isAuthenticated: !!token };
}
