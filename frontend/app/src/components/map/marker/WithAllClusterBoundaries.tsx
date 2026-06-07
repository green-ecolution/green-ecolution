import { useSuspenseQuery } from '@tanstack/react-query'
import { Polygon } from 'react-leaflet'
import { useDeferredValue } from 'react'
import { clusterBoundariesQuery } from '@/api/queries'
import { getStatusColor } from '../utils'

// PostGIS ST_AsGeoJSON of a buffered convex hull is always a Polygon geometry.
interface PolygonGeoJson {
  type: string
  coordinates: number[][][]
}

const WithAllClusterBoundaries = () => {
  const { data } = useSuspenseQuery(clusterBoundariesQuery())
  const deferred = useDeferredValue(data.data)

  return (
    <>
      {deferred.map((b) => {
        const geo = b.boundary as unknown as PolygonGeoJson
        // GeoJSON exterior ring is [lng, lat]; Leaflet wants [lat, lng].
        const ring = geo.coordinates[0].map(([lng, lat]) => [lat, lng] as [number, number])
        const color = getStatusColor(b.wateringStatus)
        return (
          <Polygon
            key={b.id}
            positions={ring}
            pathOptions={{
              color,
              fillColor: color,
              fillOpacity: 0.15,
              weight: 2,
            }}
          />
        )
      })}
    </>
  )
}

export default WithAllClusterBoundaries
