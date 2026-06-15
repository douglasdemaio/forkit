import { PinataSDK } from 'pinata-web3';

const pinata = new PinataSDK({
  pinataJwt: process.env.PINATA_JWT || '',
  pinataGateway: process.env.PINATA_GATEWAY || '',
});

export async function uploadToIPFS(
  buffer: Buffer,
  fileName: string,
  mimeType: string
): Promise<{ cid: string; url: string }> {
  const file = new File([buffer], fileName, { type: mimeType });
  const result = await pinata.upload.file(file);
  return {
    cid: result.IpfsHash,
    url: `https://gateway.pinata.cloud/ipfs/${result.IpfsHash}`,
  };
}
