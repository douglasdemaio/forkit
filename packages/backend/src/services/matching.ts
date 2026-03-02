import { haversineDistance, DeliveryZone } from './pricing-engine';

export interface DriverCandidate {
  walletAddress: string;
  zones: DeliveryZone[];
  trustScore: number;
  completedOrders: number;
}

export interface MatchResult {
  driver: DriverCandidate;
  zone: DeliveryZone;
  distanceKm: number;
  fee: number;
  score: number;
}

/**
 * Match available drivers to an order based on proximity, pricing, and trust.
 * Returns drivers sorted by composite score (lower fee + higher trust = better).
 */
export function matchDrivers(
  restaurantLat: number, restaurantLng: number,
  customerLat: number, customerLng: number,
  drivers: DriverCandidate[]
): MatchResult[] {
  const results: MatchResult[] = [];

  for (const driver of drivers) {
    for (const zone of driver.zones) {
      // Check if customer is within this zone
      const customerDist = haversineDistance(
        zone.center.lat, zone.center.lng,
        customerLat, customerLng
      );

      if (customerDist > zone.radiusKm) continue;

      const deliveryDist = haversineDistance(
        restaurantLat, restaurantLng,
        customerLat, customerLng
      );

      const fee = zone.baseFee + Math.ceil(deliveryDist * zone.perKmFee);

      // Composite score: normalize fee (lower is better) and trust (higher is better)
      // Trust is 0-10000, normalize to 0-1. Fee normalized inversely.
      const trustNorm = driver.trustScore / 10000;
      const feeNorm = 1 / (1 + fee / 1000000); // Inverse fee factor
      const score = trustNorm * 0.4 + feeNorm * 0.6;

      results.push({
        driver,
        zone,
        distanceKm: Math.round(deliveryDist * 100) / 100,
        fee,
        score,
      });
    }
  }

  // Sort by score descending (best match first)
  return results.sort((a, b) => b.score - a.score);
}
