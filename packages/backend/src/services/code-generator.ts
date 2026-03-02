import crypto from 'crypto';

const CODE_LENGTH = 6;
const CODE_CHARS = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789';

/**
 * Generate a cryptographically random alphanumeric code.
 */
export function generateCode(): string {
  const bytes = crypto.randomBytes(CODE_LENGTH);
  let code = '';
  for (let i = 0; i < CODE_LENGTH; i++) {
    code += CODE_CHARS[bytes[i] % CODE_CHARS.length];
  }
  return code;
}

/**
 * Hash a code using SHA-256 for on-chain storage.
 * Returns a 32-byte Buffer.
 */
export function hashCode(code: string): Buffer {
  return crypto.createHash('sha256').update(code).digest();
}

/**
 * Generate a code pair for an order: CODE_A (pickup) and CODE_B (delivery).
 * Returns both plaintext codes and their SHA-256 hashes.
 */
export function generateOrderCodes(): {
  codeA: string;
  codeB: string;
  codeAHash: Buffer;
  codeBHash: Buffer;
} {
  const codeA = generateCode();
  const codeB = generateCode();
  return {
    codeA,
    codeB,
    codeAHash: hashCode(codeA),
    codeBHash: hashCode(codeB),
  };
}
