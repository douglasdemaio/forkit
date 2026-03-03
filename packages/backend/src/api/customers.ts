import { Router, Response } from 'express';
import { PrismaClient } from '@prisma/client';
import { authMiddleware, AuthRequest } from '../middleware/wallet-auth';
import { generateOrderCodes } from '../services/code-generator';
import { FEE_BASIS_POINTS, DEPOSIT_BASIS_POINTS } from '../config/constants';

const router = Router();
const prisma = new PrismaClient();

/**
 * POST /api/customers/register
 */
router.post('/register', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const customer = await prisma.customer.create({
      data: {
        walletAddress: req.walletAddress!,
        metadata: req.body.metadata || {},
      },
    });
    res.status(201).json(customer);
  } catch (error: any) {
    if (error.code === 'P2002') {
      res.status(409).json({ error: 'Customer already registered' });
      return;
    }
    res.status(500).json({ error: 'Failed to register customer' });
  }
});

/**
 * GET /api/customers/restaurants
 * Browse restaurants.
 */
router.get('/restaurants', async (_req, res) => {
  try {
    const restaurants = await prisma.restaurant.findMany({
      include: {
        menuItems: { where: { available: true } },
      },
    });
    res.json(restaurants);
  } catch {
    res.status(500).json({ error: 'Failed to fetch restaurants' });
  }
});

/**
 * POST /api/customers/orders
 * Create a new order. Generates codes, computes fees, returns codes for client-side signing.
 */
router.post('/orders', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const { restaurantId, items, tokenMint, deliveryFee } = req.body;

    const customer = await prisma.customer.findUnique({
      where: { walletAddress: req.walletAddress! },
    });
    if (!customer) {
      res.status(404).json({ error: 'Customer not found' });
      return;
    }

    // Calculate food total from items
    const menuItems = await prisma.menuItem.findMany({
      where: { id: { in: items.map((i: any) => i.menuItemId) } },
    });

    let foodTotal = 0;
    for (const item of items) {
      const menuItem = menuItems.find((m) => m.id === item.menuItemId);
      if (!menuItem) {
        res.status(400).json({ error: `Menu item ${item.menuItemId} not found` });
        return;
      }
      foodTotal += Number(menuItem.price) * item.quantity;
    }

    const total = foodTotal + Number(deliveryFee);
    const protocolFee = Math.ceil((total * FEE_BASIS_POINTS) / 10000);
    const depositAmount = Math.ceil((total * DEPOSIT_BASIS_POINTS) / 10000);

    // Generate codes
    const { codeA, codeB, codeAHash, codeBHash } = generateOrderCodes();

    // Create order in database
    const onChainOrderId = BigInt(Date.now()); // Unique order ID

    const order = await prisma.order.create({
      data: {
        onChainOrderId,
        customerId: customer.id,
        restaurantId,
        items,
        tokenMint,
        foodTotal,
        deliveryFee,
        protocolFee,
        depositAmount,
        status: 'Created',
        codeAHash: codeAHash.toString('hex'),
        codeBHash: codeBHash.toString('hex'),
      },
    });

    // Return order + codes (codes only shown to relevant parties)
    res.status(201).json({
      order,
      // Client uses these hashes to call create_order on-chain
      codeAHash: Array.from(codeAHash),
      codeBHash: Array.from(codeBHash),
      // CODE_A shown to restaurant in their dashboard
      codeA,
      // CODE_B shown to customer in tracking (kept client-side)
      codeB,
      // Amounts for on-chain transaction
      onChainParams: {
        orderId: onChainOrderId.toString(),
        foodAmount: foodTotal,
        deliveryAmount: Number(deliveryFee),
        depositAmount,
        protocolFee,
        totalEscrow: total + depositAmount,
      },
    });
  } catch (error) {
    console.error('Create order error:', error);
    res.status(500).json({ error: 'Failed to create order' });
  }
});

/**
 * POST /api/customers/orders/:id/cancel
 */
router.post('/orders/:id/cancel', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const order = await prisma.order.update({
      where: { id: req.params.id, status: 'Created' },
      data: { status: 'Cancelled' },
    });
    res.json(order);
  } catch {
    res.status(400).json({ error: 'Cannot cancel order' });
  }
});

/**
 * GET /api/customers/orders
 * Order history.
 */
router.get('/orders', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const customer = await prisma.customer.findUnique({
      where: { walletAddress: req.walletAddress! },
    });
    if (!customer) {
      res.status(404).json({ error: 'Customer not found' });
      return;
    }

    const orders = await prisma.order.findMany({
      where: { customerId: customer.id },
      include: { restaurant: true },
      orderBy: { createdAt: 'desc' },
    });
    res.json(orders);
  } catch {
    res.status(500).json({ error: 'Failed to fetch orders' });
  }
});

export default router;
