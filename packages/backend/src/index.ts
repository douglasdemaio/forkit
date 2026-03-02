import express from 'express';
import cors from 'cors';
import { createServer } from 'http';
import { initializeWebSocket } from './services/notification';
import authRouter from './api/auth';
import restaurantsRouter from './api/restaurants';
import driversRouter from './api/drivers';
import customersRouter from './api/customers';

const app = express();
const server = createServer(app);
const PORT = process.env.PORT || 3001;

// Middleware
app.use(cors({ origin: process.env.FRONTEND_URL || 'http://localhost:3000' }));
app.use(express.json());

// Health check
app.get('/health', (_req, res) => {
  res.json({ status: 'ok', service: 'forkit-backend', timestamp: new Date().toISOString() });
});

// API routes
app.use('/api/auth', authRouter);
app.use('/api/restaurants', restaurantsRouter);
app.use('/api/drivers', driversRouter);
app.use('/api/customers', customersRouter);

// WebSocket
initializeWebSocket(server);

// Start
server.listen(PORT, () => {
  console.log(`🍴 ForkIt backend running on port ${PORT}`);
});

export default app;
