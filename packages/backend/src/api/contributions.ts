import { Router, Request, Response } from 'express';
import { PrismaClient } from '@prisma/client';
import crypto from 'crypto';

const router = Router();
const prisma = new PrismaClient();

/**
 * GET /api/contributions/order/:orderId
 * Get all contributions for an order (public — anyone with the order ID can see funding progress)
 */
router.get('/order/:orderId', async (req: Request, res: Response) => {
  try {
    const order = await prisma.order.findUnique({
      where: { id: req.params.orderId },
      include: {
        contributions: {
          orderBy: { createdAt: 'asc' },
        },
      },
    });

    if (!order) {
      return res.status(404).json({ error: 'Order not found' });
    }

    const fundingProgress = {
      escrowTarget: order.escrowTarget,
      escrowFunded: order.escrowFunded,
      remaining: Number(order.escrowTarget) - Number(order.escrowFunded),
      percentFunded: (Number(order.escrowFunded) / Number(order.escrowTarget)) * 100,
      contributorCount: order.contributions.length,
      contributions: order.contributions.map((c) => ({
        wallet: c.walletAddress,
        amount: c.amount,
        txSignature: c.txSignature,
        timestamp: c.createdAt,
      })),
    };

    return res.json(fundingProgress);
  } catch (error) {
    return res.status(500).json({ error: 'Failed to fetch contributions' });
  }
});

/**
 * GET /api/contributions/share/:shareLink
 * Get order details via shareable link (for friends chipping in)
 */
router.get('/share/:shareLink', async (req: Request, res: Response) => {
  try {
    const order = await prisma.order.findUnique({
      where: { shareLink: req.params.shareLink },
      include: {
        restaurant: { select: { name: true } },
        contributions: {
          orderBy: { createdAt: 'asc' },
        },
      },
    });

    if (!order) {
      return res.status(404).json({ error: 'Order not found' });
    }

    return res.json({
      orderId: order.id,
      onChainOrderId: order.onChainOrderId.toString(),
      restaurant: order.restaurant.name,
      items: order.items,
      tokenMint: order.tokenMint,
      foodTotal: order.foodTotal,
      deliveryFee: order.deliveryFee,
      escrowTarget: order.escrowTarget,
      escrowFunded: order.escrowFunded,
      remaining: Number(order.escrowTarget) - Number(order.escrowFunded),
      percentFunded: (Number(order.escrowFunded) / Number(order.escrowTarget)) * 100,
      status: order.status,
      contributions: order.contributions.map((c) => ({
        wallet: c.walletAddress.slice(0, 4) + '...' + c.walletAddress.slice(-4),
        amount: c.amount,
      })),
    });
  } catch (error) {
    return res.status(500).json({ error: 'Failed to fetch order' });
  }
});

/**
 * POST /api/contributions
 * Record a contribution (called after on-chain contribute_to_order succeeds)
 */
router.post('/', async (req: Request, res: Response) => {
  try {
    const { orderId, walletAddress, amount, txSignature } = req.body;

    // TODO: verify tx signature on-chain before recording

    const contribution = await prisma.contribution.upsert({
      where: {
        orderId_walletAddress: { orderId, walletAddress },
      },
      update: {
        amount: { increment: amount },
        txSignature,
      },
      create: {
        orderId,
        walletAddress,
        amount,
        txSignature,
      },
    });

    // Update order funding total
    const order = await prisma.order.update({
      where: { id: orderId },
      data: {
        escrowFunded: { increment: amount },
      },
    });

    // Check if fully funded
    if (Number(order.escrowFunded) >= Number(order.escrowTarget)) {
      await prisma.order.update({
        where: { id: orderId },
        data: { status: 'Funded' },
      });
    }

    return res.json({
      contribution,
      funded: Number(order.escrowFunded) >= Number(order.escrowTarget),
    });
  } catch (error) {
    return res.status(500).json({ error: 'Failed to record contribution' });
  }
});

/**
 * POST /api/contributions/generate-link/:orderId
 * Generate a shareable link for an order
 */
router.post('/generate-link/:orderId', async (req: Request, res: Response) => {
  try {
    const order = await prisma.order.findUnique({
      where: { id: req.params.orderId },
    });

    if (!order) {
      return res.status(404).json({ error: 'Order not found' });
    }

    if (order.shareLink) {
      return res.json({ shareLink: order.shareLink });
    }

    const shareLink = crypto.randomBytes(16).toString('hex');
    await prisma.order.update({
      where: { id: req.params.orderId },
      data: { shareLink },
    });

    return res.json({ shareLink });
  } catch (error) {
    return res.status(500).json({ error: 'Failed to generate link' });
  }
});

export default router;
