import { Server as SocketServer } from 'socket.io';
import { Server as HttpServer } from 'http';

let io: SocketServer | null = null;

export function initializeWebSocket(server: HttpServer): SocketServer {
  io = new SocketServer(server, {
    cors: {
      origin: process.env.FRONTEND_URL || 'http://localhost:3000',
      methods: ['GET', 'POST'],
    },
  });

  io.on('connection', (socket) => {
    console.log(`Client connected: ${socket.id}`);

    socket.on('subscribe:order', (orderId: string) => {
      socket.join(`order:${orderId}`);
      console.log(`${socket.id} subscribed to order:${orderId}`);
    });

    socket.on('unsubscribe:order', (orderId: string) => {
      socket.leave(`order:${orderId}`);
    });

    socket.on('disconnect', () => {
      console.log(`Client disconnected: ${socket.id}`);
    });
  });

  return io;
}

export type OrderEvent =
  | 'order:created'
  | 'order:accepted'
  | 'order:preparing'
  | 'order:ready'
  | 'order:picked-up'
  | 'order:delivered'
  | 'order:settled'
  | 'order:funds-released'
  | 'order:cancelled'
  | 'order:refunded'
  | 'order:disputed';

export interface FundsReleasedPayload {
  orderId: string;
  txSignature: string;
  totalReleased: number;
  restaurantReceived: number;
  driverReceived: number;
  depositRefunded: number;
  tokenSymbol: string;
}

export function emitOrderEvent(
  orderId: string,
  event: OrderEvent,
  data?: Record<string, unknown>
): void {
  if (!io) {
    console.warn('WebSocket not initialized');
    return;
  }
  io.to(`order:${orderId}`).emit(event, {
    orderId,
    timestamp: new Date().toISOString(),
    ...data,
  });
}

/** Emit the funds-released event with full payment breakdown */
export function emitFundsReleased(orderId: string, payload: FundsReleasedPayload): void {
  if (!io) {
    console.warn('WebSocket not initialized');
    return;
  }
  io.to(`order:${orderId}`).emit('order:funds-released', {
    timestamp: new Date().toISOString(),
    ...payload,
  });
  // Also emit the settled status event
  io.to(`order:${orderId}`).emit('order:settled', {
    orderId,
    timestamp: new Date().toISOString(),
    txSignature: payload.txSignature,
  });
}

/** Emit pickup event with delivery service type */
export function emitPickedUp(
  orderId: string,
  deliveryService: 'human' | 'ai',
  driverName?: string
): void {
  emitOrderEvent(orderId, 'order:picked-up', {
    deliveryService,
    driverName: driverName ?? (deliveryService === 'ai' ? 'AI Delivery Service' : 'Driver'),
  });
}
