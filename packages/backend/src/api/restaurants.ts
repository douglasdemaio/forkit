import { Router, Response } from 'express';
import { PrismaClient } from '@prisma/client';
import multer from 'multer';
import { authMiddleware, AuthRequest } from '../middleware/wallet-auth';
import { uploadToIPFS } from '../services/image-storage';
import { MAX_IMAGE_SIZE, SUPPORTED_IMAGE_TYPES } from '../config/constants';

const router = Router();
const prisma = new PrismaClient();
const upload = multer({ limits: { fileSize: MAX_IMAGE_SIZE } });

/**
 * POST /api/restaurants/register
 * Register a new restaurant profile.
 */
router.post('/register', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const { name, metadata, acceptedMints } = req.body;
    const walletAddress = req.walletAddress!;

    const restaurant = await prisma.restaurant.create({
      data: {
        walletAddress,
        name,
        metadata: metadata || {},
        acceptedMints: acceptedMints || [],
      },
    });

    res.status(201).json(restaurant);
  } catch (error: any) {
    if (error.code === 'P2002') {
      res.status(409).json({ error: 'Restaurant already registered for this wallet' });
      return;
    }
    res.status(500).json({ error: 'Failed to register restaurant' });
  }
});

/**
 * PUT /api/restaurants/profile
 * Update restaurant profile.
 */
router.put('/profile', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const restaurant = await prisma.restaurant.update({
      where: { walletAddress: req.walletAddress! },
      data: req.body,
    });
    res.json(restaurant);
  } catch {
    res.status(404).json({ error: 'Restaurant not found' });
  }
});

/**
 * POST /api/restaurants/menu
 * Add or update a menu item.
 */
router.post('/menu', authMiddleware, upload.single('image'), async (req: AuthRequest, res: Response) => {
  try {
    const restaurant = await prisma.restaurant.findUnique({
      where: { walletAddress: req.walletAddress! },
    });

    if (!restaurant) {
      res.status(404).json({ error: 'Restaurant not found' });
      return;
    }

    let imageCid = '';
    if (req.file) {
      if (!SUPPORTED_IMAGE_TYPES.includes(req.file.mimetype)) {
        res.status(400).json({ error: 'Unsupported image type' });
        return;
      }
      const result = await uploadToIPFS(req.file.buffer, req.file.originalname, req.file.mimetype);
      imageCid = result.cid;
    }

    const { name, description, price, category, menuItemId } = req.body;

    if (menuItemId) {
      // Update existing item
      const item = await prisma.menuItem.update({
        where: { id: menuItemId },
        data: { name, description, price, category, ...(imageCid && { imageCid }) },
      });
      res.json(item);
    } else {
      // Create new item
      const item = await prisma.menuItem.create({
        data: {
          restaurantId: restaurant.id,
          name,
          description,
          price,
          imageCid,
          category,
        },
      });
      res.status(201).json(item);
    }
  } catch (error) {
    res.status(500).json({ error: 'Failed to update menu' });
  }
});

/**
 * GET /api/restaurants/menu/:restaurantId
 * Fetch a restaurant's menu.
 */
router.get('/menu/:restaurantId', async (req, res) => {
  try {
    const items = await prisma.menuItem.findMany({
      where: { restaurantId: req.params.restaurantId, available: true },
      orderBy: { category: 'asc' },
    });
    res.json(items);
  } catch {
    res.status(500).json({ error: 'Failed to fetch menu' });
  }
});

/**
 * GET /api/restaurants
 * Browse restaurants.
 */
router.get('/', async (req, res) => {
  try {
    const restaurants = await prisma.restaurant.findMany({
      include: { menuItems: { where: { available: true } } },
    });
    res.json(restaurants);
  } catch {
    res.status(500).json({ error: 'Failed to fetch restaurants' });
  }
});

/**
 * GET /api/restaurants/orders
 * Fetch restaurant's orders.
 */
router.get('/orders', authMiddleware, async (req: AuthRequest, res: Response) => {
  try {
    const restaurant = await prisma.restaurant.findUnique({
      where: { walletAddress: req.walletAddress! },
    });

    if (!restaurant) {
      res.status(404).json({ error: 'Restaurant not found' });
      return;
    }

    const orders = await prisma.order.findMany({
      where: { restaurantId: restaurant.id },
      orderBy: { createdAt: 'desc' },
    });
    res.json(orders);
  } catch {
    res.status(500).json({ error: 'Failed to fetch orders' });
  }
});

export default router;
