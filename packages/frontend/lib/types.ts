export type OrderStatus =
  | 'Created'
  | 'Preparing'
  | 'ReadyForPickup'
  | 'PickedUp'
  | 'Delivered'
  | 'Settled'
  | 'Disputed'
  | 'Cancelled'
  | 'Refunded';

export type Role = 'Restaurant' | 'Driver' | 'Customer';

export interface Restaurant {
  id: string;
  walletAddress: string;
  name: string;
  metadata: RestaurantMetadata;
  acceptedMints: string[];
  menuItems?: MenuItem[];
}

export interface RestaurantMetadata {
  description?: string;
  contactEmail?: string;
  contactPhone?: string;
  address?: string;
  coordinates?: { lat: number; lng: number };
  bannerImage?: string;
  logoImage?: string;
  operatingHours?: Record<string, string>;
}

export interface MenuItem {
  id: string;
  name: string;
  description: string;
  price: number;
  imageCid: string;
  category: string;
  available: boolean;
}

export interface DeliveryZone {
  center: { lat: number; lng: number };
  radiusKm: number;
  baseFee: number;
  perKmFee: number;
  estimatedMinutes: number;
}

export interface Order {
  id: string;
  onChainOrderId: string;
  customerId: string;
  restaurantId: string;
  driverId?: string;
  items: CartItem[];
  tokenMint: string;
  foodTotal: number;
  deliveryFee: number;
  protocolFee: number;
  depositAmount: number;
  status: OrderStatus;
  createdAt: string;
  settledAt?: string;
  restaurant?: Restaurant;
}

export interface CartItem {
  menuItemId: string;
  name: string;
  quantity: number;
  price: number;
}

export interface TrustScore {
  score: number; // 0-10000
  completedOrders: number;
  totalRatings: number;
  averageRating: number;
}
