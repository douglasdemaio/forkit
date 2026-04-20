import { Router, Response } from 'express';
import { PrismaClient } from '@prisma/client';
import { authMiddleware, AuthRequest } from '../middleware/wallet-auth';
import { emitOrderEvent, emitDriverLocation } from '../services/notification';

const router = Router();
const prisma = new PrismaClient();

/**
 * POST /api/drivers/register
 */
router.post('/register', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const driver = await prisma.driver.create({
      data: {
        walletAddress: req.walletAddress!,
        metadata: req.body.metadata || {},
        zones: req.body.zones || [],
      },
    });
    res.status(201).json(driver);
  } catch (error: any) {
    if (error.code === 'P2002') {
      res.status(409).json({ error: 'Driver already registered' });
      return;
    }
    res.status(500).json({ error: 'Failed to register driver' });
  }
});

/**
 * PUT /api/drivers/zones
 * Update delivery zones and pricing.
 */
router.put('/zones', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const driver = await prisma.driver.update({
      where: { walletAddress: req.walletAddress! },
      data: { zones: req.body.zones },
    });
    res.json(driver);
  } catch {
    res.status(404).json({ error: 'Driver not found' });
  }
});

/**
 * GET /api/drivers/available-orders
 * Fetch orders available for pickup.
 */
router.get('/available-orders', authMiddleware, async (_req: AuthRequest, res: Response) => {
  try {
    const orders = await prisma.order.findMany({
      where: { status: 'Funded', driverId: null },
      include: { restaurant: true },
      orderBy: { createdAt: 'asc' },
    });
    res.json(orders);
  } catch {
    res.status(500).json({ error: 'Failed to fetch orders' });
  }
});

/**
 * POST /api/drivers/orders/:id/accept
 */
router.post('/orders/:id/accept', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const driver = await prisma.driver.findUnique({
      where: { walletAddress: req.walletAddress! },
    });

    if (!driver) {
      res.status(404).json({ error: 'Driver not found' });
      return;
    }

    const order = await prisma.order.update({
      where: { id: req.params.id, status: 'Funded' },
      data: { driverId: driver.id, status: 'Preparing' },
    });

    emitOrderEvent(order.id, 'order:accepted');

    res.json(order);
  } catch {
    res.status(400).json({ error: 'Failed to accept order' });
  }
});

/**
 * PUT /api/drivers/orders/:id/location
 * Stream driver GPS coordinates to the order room via Socket.IO.
 */
router.put('/orders/:id/location', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const { lat, lng } = req.body;

    if (
      typeof lat !== 'number' ||
      typeof lng !== 'number' ||
      lat < -90 || lat > 90 ||
      lng < -180 || lng > 180
    ) {
      res.status(400).json({ error: 'Invalid coordinates: lat must be -90..90 and lng must be -180..180' });
      return;
    }

    const driver = await prisma.driver.findUnique({
      where: { walletAddress: req.walletAddress! },
    });

    if (!driver) {
      res.status(404).json({ error: 'Driver not found' });
      return;
    }

    const order = await prisma.order.findUnique({
      where: { id: req.params.id },
    });

    if (!order) {
      res.status(404).json({ error: 'Order not found' });
      return;
    }

    if (order.driverId !== driver.id) {
      res.status(403).json({ error: 'Not the assigned driver for this order' });
      return;
    }

    emitDriverLocation(req.params.id, lat, lng);

    res.json({ ok: true });
  } catch {
    res.status(500).json({ error: 'Failed to emit driver location' });
  }
});

export default router;
