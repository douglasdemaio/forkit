// Initializes the forkit_escrow protocol on devnet.
//
// What this does (4 transactions, signed by the local CLI keypair which becomes admin):
//   1. initialize_protocol(fee_basis_points = 2)
//   2. update_protocol_config(new_treasury = TREASURY_WALLET)
//   3. add_accepted_mint(USDC_MINT)
//   4. add_accepted_mint(EURC_MINT)
//
// Run from a directory whose node_modules has @solana/web3.js + bs58, e.g.
//   cd /home/douglas/forkme && node /home/douglas/forkit/scripts/init-protocol-devnet.mjs
//
// Pass --dry to print the plan without sending.

import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  SystemProgram,
  sendAndConfirmTransaction,
} from '@solana/web3.js';
import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { homedir } from 'node:os';

const RPC = process.env.SOLANA_RPC_URL || 'https://api.devnet.solana.com';
const KEYPAIR_PATH = process.env.SOLANA_KEYPAIR || `${homedir()}/.config/solana/id.json`;
const DRY = process.argv.includes('--dry');

const ESCROW_PROGRAM_ID = new PublicKey('CNUWqYhXPXszPuB8psqG2VSnwCXf1MWzT4Pztp4y8fgj');
const TREASURY_WALLET   = new PublicKey('BiP5PJuUiXPYCFx98RMCGCnRhdUVrkxSke9C6y2ZohQ9');
const USDC_MINT         = new PublicKey('4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU');
const EURC_MINT         = new PublicKey('CXk2AMBfi3TwaEL2468s6zP8xq9NxTXjp9gjMgzeUynM');
const FEE_BASIS_POINTS  = 2;

const disc = (name) =>
  createHash('sha256').update(`global:${name}`).digest().slice(0, 8);

const admin = Keypair.fromSecretKey(
  Uint8Array.from(JSON.parse(readFileSync(KEYPAIR_PATH, 'utf8')))
);
const conn = new Connection(RPC, 'confirmed');

const [protocolConfigPda] = PublicKey.findProgramAddressSync(
  [Buffer.from('protocol_config')],
  ESCROW_PROGRAM_ID
);

console.log('RPC:                ', RPC);
console.log('Admin (signer):     ', admin.publicKey.toBase58());
console.log('Escrow program:     ', ESCROW_PROGRAM_ID.toBase58());
console.log('protocol_config PDA:', protocolConfigPda.toBase58());
console.log('Treasury:           ', TREASURY_WALLET.toBase58());
console.log('USDC mint:          ', USDC_MINT.toBase58());
console.log('EURC mint:          ', EURC_MINT.toBase58());
console.log('Fee bp:             ', FEE_BASIS_POINTS);
console.log('Mode:               ', DRY ? 'dry-run' : 'broadcasting');
console.log();

const cfgInfo = await conn.getAccountInfo(protocolConfigPda);
if (cfgInfo) {
  console.log('protocol_config already exists. Skipping init; will only ensure mints are registered.');
}

async function send(label, ix) {
  if (DRY) { console.log(`[dry] ${label}`); return; }
  const tx = new Transaction().add(ix);
  const sig = await sendAndConfirmTransaction(conn, tx, [admin], { commitment: 'confirmed' });
  console.log(`✓ ${label}: ${sig}`);
}

// 1. initialize_protocol(u16 fee_basis_points)
if (!cfgInfo) {
  const data = Buffer.concat([disc('initialize_protocol'), Buffer.from(Uint16Array.of(FEE_BASIS_POINTS).buffer)]);
  await send('initialize_protocol', new TransactionInstruction({
    programId: ESCROW_PROGRAM_ID,
    keys: [
      { pubkey: protocolConfigPda,       isSigner: false, isWritable: true  },
      { pubkey: admin.publicKey,         isSigner: true,  isWritable: true  },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    data,
  }));
}

// 2. update_protocol_config(Option<u16> fee, Option<Pubkey> treasury) — only set treasury
{
  const data = Buffer.concat([
    disc('update_protocol_config'),
    Buffer.from([0]),                                      // fee = None
    Buffer.from([1]), TREASURY_WALLET.toBuffer(),          // treasury = Some(...)
  ]);
  await send('update_protocol_config (treasury)', new TransactionInstruction({
    programId: ESCROW_PROGRAM_ID,
    keys: [
      { pubkey: protocolConfigPda, isSigner: false, isWritable: true },
      { pubkey: admin.publicKey,   isSigner: true,  isWritable: false },
    ],
    data,
  }));
}

// 3 + 4. add_accepted_mint(USDC), add_accepted_mint(EURC). Idempotent: skip if already present.
async function addMint(label, mint) {
  // Re-read config to check whether the mint is already in accepted_mints
  const info = await conn.getAccountInfo(protocolConfigPda);
  if (info) {
    const d = info.data;
    const vecLen = d.readUInt32LE(8 + 32 + 32 + 2);
    const base = 8 + 32 + 32 + 2 + 4;
    for (let i = 0; i < vecLen; i++) {
      const m = new PublicKey(d.slice(base + i * 32, base + (i + 1) * 32));
      if (m.equals(mint)) { console.log(`${label} already accepted, skipping`); return; }
    }
  }
  await send(`add_accepted_mint (${label})`, new TransactionInstruction({
    programId: ESCROW_PROGRAM_ID,
    keys: [
      { pubkey: protocolConfigPda, isSigner: false, isWritable: true  },
      { pubkey: mint,              isSigner: false, isWritable: false },
      { pubkey: admin.publicKey,   isSigner: true,  isWritable: false },
    ],
    data: disc('add_accepted_mint'),
  }));
}

await addMint('USDC', USDC_MINT);
await addMint('EURC', EURC_MINT);

console.log('\nDone.');
