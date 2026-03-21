'use client';

import { useEffect, useState, useCallback } from 'react';
import { io, Socket } from 'socket.io-client';
import { API_URL } from '@/lib/constants';
import type { Order, OrderStatus, OrderReceipt, StatusEvent, FundsReleasedPayload } from '@/lib/types';

export interface OrderTrackingState {
  order: Order | null;
  currentStatus: OrderStatus | null;
  statusHistory: StatusEvent[];
  receipt: OrderReceipt | null;
  fundsReleased: FundsReleasedPayload | null;
  isLoading: boolean;
  error: string | null;
}

const STATUS_NOTES: Partial<Record<OrderStatus, string>> = {
  Created: 'Order placed and payment locked in escrow',
  Preparing: 'Restaurant accepted and is cooking your order',
  ReadyForPickup: 'Order is packed and ready for collection',
  PickedUp: 'Driver has collected your order',
  Delivered: 'Order delivered to you',
  Settled: 'Funds released on-chain to restaurant and driver',
  Cancelled: 'Order cancelled',
  Disputed: 'Dispute opened – escrow held pending review',
  Refunded: 'Funds returned to your wallet',
};

export function useOrderTracking(orderId: string | null, authToken?: string): OrderTrackingState {
  const [order, setOrder] = useState<Order | null>(null);
  const [currentStatus, setCurrentStatus] = useState<OrderStatus | null>(null);
  const [statusHistory, setStatusHistory] = useState<StatusEvent[]>([]);
  const [receipt, setReceipt] = useState<OrderReceipt | null>(null);
  const [fundsReleased, setFundsReleased] = useState<FundsReleasedPayload | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const pushEvent = useCallback((event: StatusEvent) => {
    setStatusHistory((prev) => {
      // Avoid duplicate statuses
      if (prev.some((e) => e.status === event.status)) return prev;
      return [...prev, event];
    });
    setCurrentStatus(event.status);
  }, []);

  // Fetch initial order + receipt from API
  useEffect(() => {
    if (!orderId) return;

    const headers: Record<string, string> = { 'Content-Type': 'application/json' };
    if (authToken) headers['Authorization'] = `Bearer ${authToken}`;

    setIsLoading(true);

    Promise.all([
      fetch(`${API_URL}/api/customers/orders/${orderId}`, { headers }).then((r) =>
        r.ok ? r.json() : null
      ),
      fetch(`${API_URL}/api/customers/orders/${orderId}/receipt`, { headers }).then((r) =>
        r.ok ? r.json() : null
      ),
    ])
      .then(([orderData, receiptData]) => {
        if (orderData) {
          setOrder(orderData);
          const initialStatus = orderData.status as OrderStatus;
          setCurrentStatus(initialStatus);
          // Seed history with the initial state from the server
          setStatusHistory([
            {
              status: 'Created',
              timestamp: orderData.createdAt,
              note: STATUS_NOTES['Created'],
            },
            // If status is past Created, add the current status too
            ...(initialStatus !== 'Created'
              ? [
                  {
                    status: initialStatus,
                    timestamp: orderData.settledAt || orderData.createdAt,
                    txSignature: orderData.settleTxSignature,
                    deliveryService: orderData.deliveryService,
                    note: STATUS_NOTES[initialStatus],
                  } as StatusEvent,
                ]
              : []),
          ]);
        }
        if (receiptData) setReceipt(receiptData);
      })
      .catch(() => setError('Failed to load order details'))
      .finally(() => setIsLoading(false));
  }, [orderId, authToken]);

  // WebSocket subscription for live events
  useEffect(() => {
    if (!orderId) return;

    const socket: Socket = io(API_URL, {
      auth: authToken ? { token: authToken } : undefined,
    });

    socket.on('connect', () => {
      socket.emit('subscribe:order', orderId);
    });

    // Order status events → update timeline
    const statusMap: Record<string, OrderStatus> = {
      'order:created': 'Created',
      'order:accepted': 'Preparing',
      'order:preparing': 'Preparing',
      'order:ready': 'ReadyForPickup',
      'order:picked-up': 'PickedUp',
      'order:delivered': 'Delivered',
      'order:settled': 'Settled',
      'order:cancelled': 'Cancelled',
      'order:disputed': 'Disputed',
      'order:refunded': 'Refunded',
    };

    Object.entries(statusMap).forEach(([event, status]) => {
      socket.on(event, (payload: Record<string, unknown>) => {
        pushEvent({
          status,
          timestamp: (payload?.timestamp as string) || new Date().toISOString(),
          deliveryService: payload?.deliveryService as 'human' | 'ai' | undefined,
          note: STATUS_NOTES[status],
        });
      });
    });

    // Funds released: update receipt inline
    socket.on('order:funds-released', (payload: FundsReleasedPayload) => {
      setFundsReleased(payload);
      pushEvent({
        status: 'Settled',
        timestamp: new Date().toISOString(),
        txSignature: payload.txSignature,
        note: STATUS_NOTES['Settled'],
      });
      // Update receipt with settlement tx
      setReceipt((prev) =>
        prev
          ? {
              ...prev,
              status: 'Settled',
              settleTxSignature: payload.txSignature,
              settledAt: new Date().toISOString(),
              depositRefunded: payload.depositRefunded,
              netPaid: prev.totalCharged - payload.depositRefunded,
            }
          : prev
      );
    });

    return () => {
      socket.emit('unsubscribe:order', orderId);
      socket.disconnect();
    };
  }, [orderId, authToken, pushEvent]);

  return { order, currentStatus, statusHistory, receipt, fundsReleased, isLoading, error };
}
