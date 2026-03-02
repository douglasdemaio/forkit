'use client';

import { useCallback } from 'react';
import { useConnection, useWallet } from '@solana/wallet-adapter-react';
import { PublicKey, SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from '@solana/spl-token';

/**
 * Hook for interacting with the ForkIt escrow program.
 * Placeholder — requires the actual IDL and program ID after deployment.
 */
export function useEscrow() {
  const { connection } = useConnection();
  const { publicKey, sendTransaction } = useWallet();

  const getOrderPDA = useCallback(
    (orderId: bigint, programId: PublicKey) => {
      return PublicKey.findProgramAddressSync(
        [Buffer.from('order'), Buffer.from(orderId.toString())],
        programId
      );
    },
    []
  );

  const getEscrowVaultPDA = useCallback(
    (orderId: bigint, programId: PublicKey) => {
      return PublicKey.findProgramAddressSync(
        [Buffer.from('escrow_vault'), Buffer.from(orderId.toString())],
        programId
      );
    },
    []
  );

  return {
    getOrderPDA,
    getEscrowVaultPDA,
    connection,
    publicKey,
    sendTransaction,
  };
}
