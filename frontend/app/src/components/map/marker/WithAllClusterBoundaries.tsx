import { useSuspenseQuery } from '@tanstack/react-query'
import { Polygon } from 'react-leaflet'
import { memo } from 'react'
import { clusterBoundariesQuery } from '@/api/queries'
import useStore from '@/store/store'
import { getStatusColor } from '../utils'

interface PolygonGeoJson {
  type: string
  coordinates: number[][][]
}

const metersPerPixel = (lat: number, zoom: number) =>
  (156543.03392 * Math.cos((lat * Math.PI) / 180)) / 2 ** zoom

const WithAllClusterBoundaries = memo(() => {
  const { data } = useSuspenseQuery(clusterBoundariesQuery())
  const zoom = useStore((s) => s.mapZoom)

  return (
    <>
      {data.data.map((b) => {
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
        const mpp = metersPerPixel(midLat, zoom)
        const widthPx = ((maxLng - minLng) * 111320 * Math.cos((midLat * Math.PI) / 180)) / mpp
        const heightPx = ((maxLat - minLat) * 110540) / mpp
        const sizePx = Math.max(widthPx, heightPx)

        const weight = Math.min(12, Math.max(2, Math.round(12 - (sizePx - 40) * 0.1)))
        const opacity = weight <= 4 ? 0.9 : 0.5
        const ring = coords.map(([lng, lat]) => [lat, lng] as [number, number])
        const color = getStatusColor(b.wateringStatus)

        return (
          <Polygon
            key={b.id}
            positions={ring}
            pathOptions={{
              className: 'cluster-boundary',
              color,
              fillColor: color,
              fillOpacity: 0.15,
              weight,
              opacity,
              lineJoin: 'round',
              lineCap: 'round',
            }}
          />
        )
      })}
    </>
  )
})

WithAllClusterBoundaries.displayName = 'WithAllClusterBoundaries'

export default WithAllClusterBoundaries
