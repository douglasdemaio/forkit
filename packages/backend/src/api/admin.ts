import { Router, Response } from 'express';
import { PrismaClient } from '@prisma/client';
import { authMiddleware, AuthRequest } from '../middleware/wallet-auth';

const router = Router();
const prisma = new PrismaClient();

const ADMIN_WALLET = process.env.ADMIN_WALLET || '';

function adminOnly(req: AuthRequest, res: Response, next: Function): void {
  if (!ADMIN_WALLET || req.walletAddress !== ADMIN_WALLET) {
    res.status(403).json({ error: 'Admin access required' });
    return;
  }
  next();
}

// GET /api/admin/disputes
router.get('/disputes', authMiddleware, adminOnly, async (_req, res) => {
  try {
    const orders = await prisma.order.findMany({
      where: { status: 'Disputed' },
      include: { restaurant: true, contributions: true },
      orderBy: { createdAt: 'asc' },
    });
    res.json(orders);
  } catch {
    res.status(500).json({ error: 'Failed to fetch disputes' });
  }
});

// PATCH /api/admin/disputes/:orderId/resolve
// Called after admin signs resolve_dispute on-chain, to mirror state to Postgres
router.patch('/disputes/:orderId/resolve', authMiddleware, adminOnly, async (req: AuthRequest, res: Response) => {
  try {
    const { resolution, txSignature } = req.body;
    const valid = ['RefundCustomer', 'PayRestaurantAndDriver', 'Split'];
    if (!valid.includes(resolution)) {
      res.status(400).json({ error: 'Invalid resolution' });
      return;
    }
    const order = await prisma.order.update({
      where: { id: req.params.orderId as string, status: 'Disputed' },
      data: { status: 'Refunded' },
    });

    res.json({ order, resolution, txSignature });
  } catch {
    res.status(400).json({ error: 'Failed to resolve dispute' });
  }
});

export default router;
