import { Suspense, useCallback, useEffect, useState } from 'react'
import { useNavigate } from '@tanstack/react-router'
import { useSuspenseQuery } from '@tanstack/react-query'
import { Loading } from '@green-ecolution/ui'
import type { WateringPlan } from '@/api/backendApi'
import { clusterMarkersQuery, wateringPlanRouteQuery } from '@/api/queries'
import useStore from '@/store/store'
import MapPreview from '@/components/map-gl/MapPreview'
import { useMaplibreMap } from '@/components/map-gl/MapContext'
import useClusterBoundaryLayer from '@/components/map-gl/layers/useClusterBoundaryLayer'
import useClusterMarkerLayer from '@/components/map-gl/layers/useClusterMarkerLayer'
import useRouteLayer from '@/components/map-gl/layers/useRouteLayer'
import { isMapAlive } from '@/components/map-gl/mapReady'
import ClusterPanel from '@/components/map/cluster-panel/ClusterPanel'

interface WateringPlanPreviewRouteProps {
  wateringPlan: WateringPlan
}

const RoutePreviewLayers = ({
  planId,
  clusterIds,
  onSelectCluster,
}: {
  planId: string
  clusterIds: string[]
  onSelectCluster: (id: string) => void
}) => {
  const map = useMaplibreMap()
  const { data: markers } = useSuspenseQuery(clusterMarkersQuery())
  const { data: route } = useSuspenseQuery(wateringPlanRouteQuery(planId))

  const routeCoordinates = route?.geometry.coordinates as [number, number][] | undefined

  useClusterBoundaryLayer({ onBoundaryClick: onSelectCluster, clusterIds })
  useRouteLayer({ coordinates: routeCoordinates })
  useClusterMarkerLayer({ onClusterClick: onSelectCluster, clusterIds, flyToOnClick: false })

  useEffect(() => {
    if (!isMapAlive(map)) return
    let points: [number, number][]
    if (routeCoordinates?.length) {
      points = routeCoordinates
    } else {
      const ids = new Set(clusterIds)
      points = markers.data
        .filter((c) => ids.has(c.id))
        .map((c) => [c.longitude, c.latitude] as [number, number])
    }
    if (points.length === 0) return
    let minLng = points[0][0]
    let maxLng = points[0][0]
    let minLat = points[0][1]
    let maxLat = points[0][1]
    for (const [lng, lat] of points) {
      if (lng < minLng) minLng = lng
      if (lng > maxLng) maxLng = lng
      if (lat < minLat) minLat = lat
      if (lat > maxLat) maxLat = lat
    }
    map.fitBounds(
      [
        [minLng, minLat],
        [maxLng, maxLat],
      ],
      { padding: 64, maxZoom: 16 },
    )
  }, [map, markers, clusterIds, routeCoordinates])

  return null
}

const WateringPlanPreviewRoute = ({ wateringPlan }: WateringPlanPreviewRouteProps) => {
  const [centerLat, centerLng] = useStore.getState().mapCenter
  const clusterIds = wateringPlan.treeclusters.map((tc) => tc.id)
  const [selectedClusterId, setSelectedClusterId] = useState<string | null>(null)
  const navigate = useNavigate()

  const handleSelectCluster = useCallback((id: string) => {
    setSelectedClusterId(id)
  }, [])

  const handleClosePanel = useCallback(() => {
    setSelectedClusterId(null)
  }, [])

  const handleOpenDashboard = useCallback(() => {
    if (!selectedClusterId) return
    navigate({
      to: '/treecluster/$treeclusterId',
      params: { treeclusterId: selectedClusterId },
    }).catch((error) => console.error('Navigation failed:', error))
  }, [navigate, selectedClusterId])

  return (
    <MapPreview
      center={[centerLng, centerLat]}
      zoom={13}
      interactive
      className="h-[40rem]"
      ariaLabel="Karte mit der Route und den Bewässerungsgruppen des Plans"
    >
      <Suspense fallback={<Loading className="justify-center" label="Lade Karte..." />}>
        <RoutePreviewLayers
          planId={wateringPlan.id}
          clusterIds={clusterIds}
          onSelectCluster={handleSelectCluster}
        />
      </Suspense>
      {selectedClusterId && (
        <Suspense fallback={null}>
          <ClusterPanel
            key={selectedClusterId}
            clusterId={selectedClusterId}
            onClose={handleClosePanel}
            onOpenDashboard={handleOpenDashboard}
          />
        </Suspense>
      )}
    </MapPreview>
  )
}

export default WateringPlanPreviewRoute
