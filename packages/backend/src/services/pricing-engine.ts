/**
 * Haversine formula — calculate distance between two coordinates in km.
 */
export function haversineDistance(
  lat1: number, lng1: number,
  lat2: number, lng2: number
): number {
  const R = 6371; // Earth's radius in km
  const dLat = toRad(lat2 - lat1);
  const dLng = toRad(lng2 - lng1);
  const a =
    Math.sin(dLat / 2) ** 2 +
    Math.cos(toRad(lat1)) * Math.cos(toRad(lat2)) * Math.sin(dLng / 2) ** 2;
  const c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1 - a));
  return R * c;
}

function toRad(deg: number): number {
  return (deg * Math.PI) / 180;
}

export interface DeliveryZone {
  center: { lat: number; lng: number };
  radiusKm: number;
  baseFee: number;     // stablecoin units
  perKmFee: number;    // stablecoin units per km
  estimatedMinutes: number;
}

/**
 * Calculate delivery fee for a given restaurant-to-customer distance
 * using a driver's zone configuration.
 */
export function calculateDeliveryFee(
  restaurantLat: number, restaurantLng: number,
  customerLat: number, customerLng: number,
  zone: DeliveryZone
): { fee: number; distanceKm: number; estimatedMinutes: number } | null {
  const distanceKm = haversineDistance(restaurantLat, restaurantLng, customerLat, customerLng);

  // Check if delivery is within zone radius
  const distFromCenter = haversineDistance(
    zone.center.lat, zone.center.lng,
    customerLat, customerLng
  );

  if (distFromCenter > zone.radiusKm) {
    return null; // Outside delivery zone
  }

  const fee = zone.baseFee + Math.ceil(distanceKm * zone.perKmFee);
  const estimatedMinutes = Math.ceil(zone.estimatedMinutes * (distanceKm / zone.radiusKm));

  return { fee, distanceKm: Math.round(distanceKm * 100) / 100, estimatedMinutes };
}

/**
 * Find the best matching driver zone for a delivery.
 */
export function findBestZone(
  restaurantLat: number, restaurantLng: number,
  customerLat: number, customerLng: number,
  zones: DeliveryZone[]
): { zone: DeliveryZone; fee: number; distanceKm: number; estimatedMinutes: number } | null {
  let best: { zone: DeliveryZone; fee: number; distanceKm: number; estimatedMinutes: number } | null = null;

  for (const zone of zones) {
    const result = calculateDeliveryFee(restaurantLat, restaurantLng, customerLat, customerLng, zone);
    if (result && (!best || result.fee < best.fee)) {
      best = { zone, ...result };
    }
  }

  return best;
}
