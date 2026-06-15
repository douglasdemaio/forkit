import crypto from 'crypto';

export function generateOrderCodes(): {
  codeA: string;
  codeB: string;
  codeAHash: Buffer;
  codeBHash: Buffer;
} {
  const codeA = crypto.randomBytes(4).toString('hex').toUpperCase();
  const codeB = crypto.randomBytes(4).toString('hex').toUpperCase();

  const codeAHash = crypto.createHash('sha256').update(codeA).digest();
  const codeBHash = crypto.createHash('sha256').update(codeB).digest();

  return { codeA, codeB, codeAHash, codeBHash };
}
