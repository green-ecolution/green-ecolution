import { useEffect } from 'react'
import { useSuspenseQuery } from '@tanstack/react-query'
import type { LngLatBoundsLike } from 'maplibre-gl'
import type { ClusterMarkerResponse } from '@green-ecolution/backend-client'
import { wateringPlanQueries } from '@/api/queries'
import { useMaplibreMap } from '@/components/map-gl/MapContext'
import { isMapAlive } from '@/components/map-gl/mapReady'
import { clusterQueries } from '../../api/queries'

export const stopBounds = (
  markers: ClusterMarkerResponse[],
  stopIds: string[],
): LngLatBoundsLike | undefined => {
  const ids = new Set(stopIds)
  const points = markers.filter((m) => ids.has(m.id)).map((m) => [m.longitude, m.latitude])
  if (points.length === 0) return undefined
  const lngs = points.map(([lng]) => lng)
  const lats = points.map(([, lat]) => lat)
  return [
    [Math.min(...lngs), Math.min(...lats)],
    [Math.max(...lngs), Math.max(...lats)],
  ]
}

/** The core frames the map on the route; a plan without one is framed on its clusters. */
const useFitStopsWithoutRoute = (planId: string, stopIds: string[]) => {
  const map = useMaplibreMap()
  const { data: route } = useSuspenseQuery(wateringPlanQueries.route(planId))
  const { data: markers } = useSuspenseQuery(clusterQueries.markers())
  const hasRoute = Boolean(route?.geometry.coordinates.length)

  useEffect(() => {
    if (hasRoute || !isMapAlive(map)) return
    const bounds = stopBounds(markers.data, stopIds)
    if (bounds) map.fitBounds(bounds, { padding: 64, maxZoom: 16 })
  }, [map, hasRoute, markers, stopIds])
}

export default useFitStopsWithoutRoute
