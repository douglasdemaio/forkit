'use client';

import { useEffect, useState } from 'react';
import { io, Socket } from 'socket.io-client';
import { API_URL } from '@/lib/constants';
import type { OrderStatus } from '@/lib/types';

export function useOrderStatus(orderId: string | null) {
  const [status, setStatus] = useState<OrderStatus | null>(null);
  const [socket, setSocket] = useState<Socket | null>(null);

  useEffect(() => {
    if (!orderId) return;

    const s = io(API_URL);
    setSocket(s);

    s.on('connect', () => {
      s.emit('subscribe:order', orderId);
    });

    const events: Record<string, OrderStatus> = {
      'order:accepted': 'Preparing',
      'order:preparing': 'Preparing',
      'order:ready': 'ReadyForPickup',
      'order:picked-up': 'PickedUp',
      'order:delivered': 'Settled',
      'order:cancelled': 'Cancelled',
      'order:disputed': 'Disputed',
    };

    Object.entries(events).forEach(([event, newStatus]) => {
      s.on(event, () => setStatus(newStatus));
    });

    return () => {
      s.emit('unsubscribe:order', orderId);
      s.disconnect();
    };
  }, [orderId]);

  return { status, socket };
}
