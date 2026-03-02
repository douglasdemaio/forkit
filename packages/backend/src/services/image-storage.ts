/**
 * Image storage service — uploads to IPFS via Pinata.
 * Stub implementation for MVP; replace with actual Pinata SDK calls.
 */

export interface UploadResult {
  cid: string;
  url: string;
}

const PINATA_API_KEY = process.env.PINATA_API_KEY || '';
const PINATA_SECRET_KEY = process.env.PINATA_SECRET_KEY || '';
const PINATA_GATEWAY = 'https://gateway.pinata.cloud/ipfs';

/**
 * Upload an image buffer to IPFS via Pinata.
 */
export async function uploadToIPFS(
  buffer: Buffer,
  filename: string,
  _mimeType: string
): Promise<UploadResult> {
  if (!PINATA_API_KEY || !PINATA_SECRET_KEY) {
    // Dev fallback: return a placeholder CID
    const fakeCid = `Qm${Buffer.from(filename).toString('hex').slice(0, 44)}`;
    console.warn('Pinata not configured, using placeholder CID:', fakeCid);
    return { cid: fakeCid, url: `${PINATA_GATEWAY}/${fakeCid}` };
  }

  // Production: use Pinata SDK or API
  const FormData = (await import('form-data')).default;
  const form = new FormData();
  form.append('file', buffer, { filename });

  const response = await fetch('https://api.pinata.cloud/pinning/pinFileToIPFS', {
    method: 'POST',
    headers: {
      pinata_api_key: PINATA_API_KEY,
      pinata_secret_api_key: PINATA_SECRET_KEY,
      ...form.getHeaders(),
    },
    body: form as unknown as BodyInit,
  });

  if (!response.ok) {
    throw new Error(`Pinata upload failed: ${response.statusText}`);
  }

  const data = (await response.json()) as { IpfsHash: string };
  return {
    cid: data.IpfsHash,
    url: `${PINATA_GATEWAY}/${data.IpfsHash}`,
  };
}
