import { Router, Response } from 'express';
import { PrismaClient } from '@prisma/client';
import rateLimit from 'express-rate-limit';
import { authMiddleware, AuthRequest } from '../middleware/wallet-auth';
import { generateOrderCodes } from '../services/code-generator';
import { FEE_BASIS_POINTS, DEPOSIT_BASIS_POINTS } from '../config/constants';
import { emitOrderEvent, emitFundsReleased } from '../services/notification';

const router = Router();
const prisma = new PrismaClient();

const apiLimiter = rateLimit({
  windowMs: 15 * 60 * 1000, // 15 minutes
  max: 100, // max 100 requests per IP per window
  standardHeaders: true,
  legacyHeaders: false,
});

router.use(apiLimiter);

// Known token metadata (mint → symbol/sign)
const TOKEN_META: Record<string, { symbol: string; currencySign: string }> = {
  '4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU': { symbol: 'USDC', currencySign: '$' },
  EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v: { symbol: 'USDC', currencySign: '$' },
  HzwqbKZw8HxMN6bF2yFZNrht3c2iXXzpKcFu7uBEDKtr: { symbol: 'EURC', currencySign: '€' },
  CXk2AMBfi3TwaEL2468s6zP8xq9NxTXjp9gjMgzeUynM: { symbol: 'EURC', currencySign: '€' },
};

function tokenMeta(mint: string) {
  return TOKEN_META[mint] ?? { symbol: 'USDC', currencySign: '$' };
}

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
    const escrowTarget = total + depositAmount;

    // Generate codes
    const { codeA, codeB, codeAHash, codeBHash } = generateOrderCodes();

    // Create order in database
    const onChainOrderId = BigInt(Date.now());

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
        escrowTarget,
        status: 'Created',
        codeAHash: codeAHash.toString('hex'),
        codeBHash: codeBHash.toString('hex'),
      },
    });

    emitOrderEvent(order.id, 'order:created');

    // Return order + codes (codes only shown to relevant parties)
    res.status(201).json({
      order,
      codeAHash: Array.from(codeAHash),
      codeBHash: Array.from(codeBHash),
      // CODE_A shown to restaurant in their dashboard
      codeA,
      // CODE_B shown to customer for delivery verification
      codeB,
      onChainParams: {
        orderId: onChainOrderId.toString(),
        foodAmount: foodTotal,
        deliveryAmount: Number(deliveryFee),
        depositAmount,
        protocolFee,
        totalEscrow: escrowTarget,
      },
    });
  } catch (error) {
    console.error('Create order error:', error);
    res.status(500).json({ error: 'Failed to create order' });
  }
});

/**
 * GET /api/customers/orders/:id
 * Full order details for the tracking page.
 */
router.get('/orders/:id', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const order = await prisma.order.findUnique({
      where: { id: req.params.id },
      include: { restaurant: true, driver: true },
    });
    if (!order) {
      res.status(404).json({ error: 'Order not found' });
      return;
    }
    res.json(order);
  } catch {
    res.status(500).json({ error: 'Failed to fetch order' });
  }
});

/**
 * GET /api/customers/orders/:id/receipt
 * Returns a structured receipt for a completed (or in-progress) order.
 */
router.get('/orders/:id/receipt', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const order = await prisma.order.findUnique({
      where: { id: req.params.id },
      include: { restaurant: true },
    });
    if (!order) {
      res.status(404).json({ error: 'Order not found' });
      return;
    }

    const { symbol, currencySign } = tokenMeta(order.tokenMint);
    const foodTotal = Number(order.foodTotal);
    const deliveryFee = Number(order.deliveryFee);
    const protocolFee = Number(order.protocolFee);
    const depositAmount = Number(order.depositAmount);
    const totalCharged = foodTotal + deliveryFee + protocolFee + depositAmount;
    const isSettled = order.status === 'Settled' || order.status === 'Delivered';
    const depositRefunded = isSettled ? depositAmount : 0;
    const netPaid = totalCharged - depositRefunded;

    res.json({
      orderId: order.id,
      onChainOrderId: order.onChainOrderId.toString(),
      restaurantName: order.restaurant?.name ?? 'Restaurant',
      items: order.items,
      tokenMint: order.tokenMint,
      tokenSymbol: symbol,
      currencySign,
      foodTotal,
      deliveryFee,
      protocolFee,
      depositAmount,
      depositRefunded,
      totalCharged,
      netPaid,
      status: order.status,
      createdAt: order.createdAt.toISOString(),
      settledAt: order.settledAt?.toISOString(),
      settleTxSignature: (order as any).settleTxSignature ?? null,
      deliveryService: (order as any).deliveryService ?? null,
    });
  } catch (error) {
    console.error('Receipt error:', error);
    res.status(500).json({ error: 'Failed to generate receipt' });
  }
});

/**
 * PATCH /api/customers/orders/:id/settle
 * Called by the backend/on-chain listener when escrow is settled.
 * Records the settlement tx and emits the funds-released WebSocket event.
 */
router.patch('/orders/:id/settle', async (req, res: Response) => {
  try {
    const { txSignature, deliveryService } = req.body;

    const order = await prisma.order.update({
      where: { id: req.params.id },
      data: {
        status: 'Settled',
        settledAt: new Date(),
        ...(txSignature && { settleTxSignature: txSignature }),
        ...(deliveryService && { deliveryService }),
      },
      include: { restaurant: true },
    });

    const { symbol, currencySign } = tokenMeta(order.tokenMint);
    const foodTotal = Number(order.foodTotal);
    const deliveryFee = Number(order.deliveryFee);
    const protocolFee = Number(order.protocolFee);
    const depositAmount = Number(order.depositAmount);
    const totalEscrow = foodTotal + deliveryFee + protocolFee;

    emitFundsReleased(order.id, {
      orderId: order.id,
      txSignature: txSignature ?? '',
      totalReleased: totalEscrow,
      restaurantReceived: foodTotal,
      driverReceived: deliveryFee,
      depositRefunded: depositAmount,
      tokenSymbol: symbol,
    });

    res.json({ success: true, order });
  } catch (error) {
    console.error('Settle error:', error);
    res.status(500).json({ error: 'Failed to settle order' });
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
    emitOrderEvent(order.id, 'order:cancelled');
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
