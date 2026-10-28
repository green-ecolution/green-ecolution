const EARTH_RADIUS_METERS = 6_371_000

interface Point {
  latitude: number
  longitude: number
}

const radians = (degrees: number) => (degrees * Math.PI) / 180

export const distanceMeters = (a: Point, b: Point): number => {
  const dLat = radians(b.latitude - a.latitude)
  const dLng = radians(b.longitude - a.longitude)
  const h =
    Math.sin(dLat / 2) ** 2 +
    Math.cos(radians(a.latitude)) * Math.cos(radians(b.latitude)) * Math.sin(dLng / 2) ** 2
  return 2 * EARTH_RADIUS_METERS * Math.asin(Math.sqrt(h))
}
