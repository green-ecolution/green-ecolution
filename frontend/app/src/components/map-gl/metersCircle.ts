import type { Feature, Polygon } from 'geojson'

const EARTH_RADIUS_M = 6378137

// Approximates a geographic circle of `meters` radius as a polygon so MapLibre
// (which has no metric-radius primitive) can render an accuracy ring that scales
// correctly with zoom.
export const metersCircle = (
  lng: number,
  lat: number,
  meters: number,
  steps = 64,
): Feature<Polygon> => {
  const dLat = (meters / EARTH_RADIUS_M) * (180 / Math.PI)
  const dLng = dLat / Math.cos((lat * Math.PI) / 180)
  const ring: [number, number][] = []
  for (let i = 0; i <= steps; i++) {
    const theta = (i / steps) * 2 * Math.PI
    ring.push([lng + dLng * Math.cos(theta), lat + dLat * Math.sin(theta)])
  }
  return { type: 'Feature', geometry: { type: 'Polygon', coordinates: [ring] }, properties: {} }
}
