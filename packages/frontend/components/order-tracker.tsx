'use client';

import type { OrderStatus, StatusEvent, DeliveryService } from '@/lib/types';

// --- Step definitions ---

interface Step {
  status: OrderStatus[];
  label: string;
  description: (event?: StatusEvent) => string;
  icon: React.ReactNode;
}

const CheckIcon = () => (
  <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2.5}>
    <path strokeLinecap="round" strokeLinejoin="round" d="M5 13l4 4L19 7" />
  </svg>
);

const SpinnerIcon = () => (
  <svg className="h-5 w-5 animate-spin" fill="none" viewBox="0 0 24 24">
    <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" />
    <path
      className="opacity-75"
      fill="currentColor"
      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
    />
  </svg>
);

function deliveryLabel(service?: DeliveryService) {
  if (service === 'ai') return 'AI Delivery Service';
  if (service === 'human') return 'Delivery Driver';
  return 'Driver';
}

const STEPS: Step[] = [
  {
    status: ['Created'],
    label: 'Order Placed',
    description: () => 'Your order has been sent to the restaurant. Payment held in escrow.',
    icon: (
      <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
      </svg>
    ),
  },
  {
    status: ['Preparing'],
    label: 'Cooking',
    description: () => 'The restaurant has accepted your order and is preparing your meal.',
    icon: (
      <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M17.657 18.657A8 8 0 016.343 7.343S7 9 9 10c0-2 .5-5 2.986-7C14 5 16.09 5.777 17.656 7.343A7.975 7.975 0 0120 13a7.975 7.975 0 01-2.343 5.657z" />
        <path strokeLinecap="round" strokeLinejoin="round" d="M9.879 16.121A3 3 0 1012.015 11L11 14H9c0 .768.293 1.536.879 2.121z" />
      </svg>
    ),
  },
  {
    status: ['ReadyForPickup'],
    label: 'Ready for Pickup',
    description: () => 'Your order is packed and waiting at the restaurant.',
    icon: (
      <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
      </svg>
    ),
  },
  {
    status: ['PickedUp'],
    label: 'On the Way',
    description: (event) =>
      `${deliveryLabel(event?.deliveryService)} has picked up your order and is heading your way.`,
    icon: (
      <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M13 10V3L4 14h7v7l9-11h-7z" />
      </svg>
    ),
  },
  {
    status: ['Delivered', 'Settled'],
    label: 'Delivered & Funds Released',
    description: (event) =>
      event?.txSignature
        ? 'Order delivered! Escrow funds released to restaurant and driver on-chain.'
        : 'Order delivered! Awaiting on-chain settlement.',
    icon: (
      <svg className="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" strokeWidth={2}>
        <path strokeLinecap="round" strokeLinejoin="round" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
  },
];

// ---  helpers ---

function getStepIndex(status: OrderStatus): number {
  if (status === 'Created') return 0;
  if (status === 'Preparing') return 1;
  if (status === 'ReadyForPickup') return 2;
  if (status === 'PickedUp') return 3;
  if (status === 'Delivered' || status === 'Settled') return 4;
  return -1;
}

function formatTime(iso: string) {
  return new Date(iso).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
}

// --- Component ---

interface OrderTrackerProps {
  currentStatus: OrderStatus;
  statusHistory: StatusEvent[];
  /** Optional: show a dispute/cancel banner */
  isErrored?: boolean;
}

export function OrderTracker({ currentStatus, statusHistory, isErrored }: OrderTrackerProps) {
  const currentIndex = getStepIndex(currentStatus);

  if (isErrored || currentStatus === 'Cancelled' || currentStatus === 'Refunded' || currentStatus === 'Disputed') {
    return (
      <div className="rounded-xl border border-red-500/30 bg-red-500/10 p-5 text-center">
        <div className="mb-2 text-2xl">
          {currentStatus === 'Disputed' ? '⚖️' : currentStatus === 'Cancelled' ? '✖' : '↩'}
        </div>
        <p className="font-semibold text-red-400">
          {currentStatus === 'Disputed'
            ? 'Dispute Opened'
            : currentStatus === 'Refunded'
            ? 'Order Refunded'
            : 'Order Cancelled'}
        </p>
        <p className="mt-1 text-sm text-slate-400">
          {currentStatus === 'Disputed'
            ? 'An admin is reviewing your order. Funds remain in escrow.'
            : currentStatus === 'Refunded'
            ? 'Your funds have been returned to your wallet.'
            : 'This order was cancelled. Any held funds have been released.'}
        </p>
      </div>
    );
  }

  return (
    <div className="relative">
      {/* Vertical rail */}
      <div
        className="absolute left-5 top-5 bottom-5 w-0.5 bg-slate-700"
        aria-hidden
      />
      {/* Filled rail up to current step */}
      <div
        className="absolute left-5 top-5 w-0.5 bg-green-500 transition-all duration-700"
        style={{ height: `${(currentIndex / (STEPS.length - 1)) * 100}%` }}
        aria-hidden
      />

      <ol className="relative space-y-0">
        {STEPS.map((step, i) => {
          const isDone = i < currentIndex;
          const isActive = i === currentIndex;
          const isPending = i > currentIndex;

          // Find the matching event from history
          const event = statusHistory.find((e) => step.status.includes(e.status));

          return (
            <li key={step.label} className="flex gap-4 pb-8 last:pb-0">
              {/* Step indicator */}
              <div className="relative z-10 flex shrink-0 flex-col items-center">
                <div
                  className={`flex h-10 w-10 items-center justify-center rounded-full border-2 transition-all duration-300 ${
                    isDone
                      ? 'border-green-500 bg-green-500 text-white'
                      : isActive
                      ? 'border-green-500 bg-slate-900 text-green-400 shadow-[0_0_12px_rgba(16,185,129,0.5)]'
                      : 'border-slate-700 bg-slate-900 text-slate-600'
                  }`}
                >
                  {isDone ? <CheckIcon /> : isActive ? <SpinnerIcon /> : step.icon}
                </div>
              </div>

              {/* Step content */}
              <div className="flex-1 pt-1.5">
                <div className="flex items-center justify-between gap-2">
                  <p
                    className={`font-semibold ${
                      isDone
                        ? 'text-green-400'
                        : isActive
                        ? 'text-white'
                        : 'text-slate-500'
                    }`}
                  >
                    {step.label}
                  </p>
                  {event && (
                    <span className="shrink-0 text-xs text-slate-500">{formatTime(event.timestamp)}</span>
                  )}
                </div>

                <p className={`mt-0.5 text-sm ${isPending ? 'text-slate-600' : 'text-slate-400'}`}>
                  {step.description(event)}
                </p>

                {/* Delivery service badge */}
                {isActive && currentStatus === 'PickedUp' && event?.deliveryService && (
                  <span
                    className={`mt-2 inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium ${
                      event.deliveryService === 'ai'
                        ? 'bg-purple-500/20 text-purple-400'
                        : 'bg-blue-500/20 text-blue-400'
                    }`}
                  >
                    {event.deliveryService === 'ai' ? '🤖 AI Routing' : '🧑 Human Driver'}
                  </span>
                )}

                {/* Settlement tx link */}
                {(isDone || isActive) && step.status.includes('Settled') && event?.txSignature && (
                  <a
                    href={`https://explorer.solana.com/tx/${event.txSignature}?cluster=devnet`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="mt-2 inline-flex items-center gap-1 text-xs text-green-500 underline-offset-2 hover:underline"
                  >
                    View funds release on Solana Explorer ↗
                  </a>
                )}
              </div>
            </li>
          );
        })}
      </ol>
    </div>
  );
}
