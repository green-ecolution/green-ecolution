import { useSuspenseQuery } from '@tanstack/react-query'
import { Polygon } from 'react-leaflet'
import { memo, useMemo } from 'react'
import { clusterBoundariesQuery } from '@/api/queries'
import useStore from '@/store/store'
import { getStatusColor } from '../utils'

interface PolygonGeoJson {
  type: string
  coordinates: number[][][]
}

// Web Mercator ground resolution at zoom 0 (meters/pixel at the equator).
const MERCATOR_RESOLUTION_Z0 = 156543.03392
// WGS84 meters per degree: longitude at the equator, latitude (mean).
const METERS_PER_DEG_LNG = 111320
const METERS_PER_DEG_LAT = 110540

// Outline thickness scales inversely with on-screen size: small clusters get a
// thick stroke, large ones thin out so they don't drown the trees inside.
const MAX_STROKE_WEIGHT = 12
const MIN_STROKE_WEIGHT = 2
const FULL_WEIGHT_SIZE_PX = 40
const WEIGHT_FALLOFF_PER_PX = 0.1
// Thin strokes get raised opacity to stay visible; thick ones stay subtle.
const THIN_STROKE_MAX_WEIGHT = 4
const THIN_STROKE_OPACITY = 0.9
const THICK_STROKE_OPACITY = 0.5
const FILL_OPACITY = 0.15

const metersPerPixel = (lat: number, zoom: number) =>
  (MERCATOR_RESOLUTION_Z0 * Math.cos((lat * Math.PI) / 180)) / 2 ** zoom

interface WithAllClusterBoundariesProps {
  onClick?: (clusterId: string) => void
}

const WithAllClusterBoundaries = memo(({ onClick }: WithAllClusterBoundariesProps) => {
  const { data } = useSuspenseQuery(clusterBoundariesQuery())
  const zoom = useStore((s) => s.mapZoom)

  const boundaries = useMemo(
    () =>
      data.data.map((b) => {
        const geo = b.boundary as unknown as PolygonGeoJson
        const coords = geo.coordinates[0]

        let minLng = Infinity
        let maxLng = -Infinity
        let minLat = Infinity
        let maxLat = -Infinity
        for (const [lng, lat] of coords) {
          if (lng < minLng) minLng = lng
          if (lng > maxLng) maxLng = lng
          if (lat < minLat) minLat = lat
          if (lat > maxLat) maxLat = lat
        }

        const midLat = (minLat + maxLat) / 2
        const widthMeters =
          (maxLng - minLng) * METERS_PER_DEG_LNG * Math.cos((midLat * Math.PI) / 180)
        const heightMeters = (maxLat - minLat) * METERS_PER_DEG_LAT

        return {
          id: b.id,
          color: getStatusColor(b.wateringStatus),
          ring: coords.map(([lng, lat]) => [lat, lng] as [number, number]),
          midLat,
          extentMeters: Math.max(widthMeters, heightMeters),
        }
      }),
    [data.data],
  )

  return (
    <>
      {boundaries.map((b) => {
        const sizePx = b.extentMeters / metersPerPixel(b.midLat, zoom)
        const weight = Math.min(
          MAX_STROKE_WEIGHT,
          Math.max(
            MIN_STROKE_WEIGHT,
            Math.round(MAX_STROKE_WEIGHT - (sizePx - FULL_WEIGHT_SIZE_PX) * WEIGHT_FALLOFF_PER_PX),
          ),
        )
        const opacity =
          weight <= THIN_STROKE_MAX_WEIGHT ? THIN_STROKE_OPACITY : THICK_STROKE_OPACITY

        return (
          <Polygon
            key={b.id}
            positions={b.ring}
            eventHandlers={onClick ? { click: () => onClick(b.id) } : undefined}
            pathOptions={{
              className: onClick ? 'cluster-boundary cursor-pointer' : 'cluster-boundary',
              color: b.color,
              fillColor: b.color,
              fillOpacity: FILL_OPACITY,
              weight,
              opacity,
              lineJoin: 'round',
              lineCap: 'round',
              bubblingMouseEvents: false,
            }}
          />
        )
      })}
    </>
  )
})

WithAllClusterBoundaries.displayName = 'WithAllClusterBoundaries'

export default WithAllClusterBoundaries
