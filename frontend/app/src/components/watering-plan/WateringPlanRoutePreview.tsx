import { Suspense, useEffect } from 'react'
import { useNavigate } from '@tanstack/react-router'
import { useSuspenseQuery } from '@tanstack/react-query'
import { Loading } from '@green-ecolution/ui'
import type { WateringPlan } from '@/api/backendApi'
import { clusterMarkersQuery } from '@/api/queries'
import useStore from '@/store/store'
import MapPreview from '@/components/map-gl/MapPreview'
import { useMaplibreMap } from '@/components/map-gl/MapContext'
import useClusterBoundaryLayer from '@/components/map-gl/layers/useClusterBoundaryLayer'
import useClusterMarkerLayer from '@/components/map-gl/layers/useClusterMarkerLayer'
import { isMapAlive } from '@/components/map-gl/mapReady'

interface WateringPlanPreviewRouteProps {
  wateringPlan: WateringPlan
}

const RoutePreviewLayers = ({ clusterIds }: { clusterIds: string[] }) => {
  const map = useMaplibreMap()
  const navigate = useNavigate()
  const { data: markers } = useSuspenseQuery(clusterMarkersQuery())

  const navToCluster = (id: string) =>
    navigate({ to: '/treecluster/$treeclusterId', params: { treeclusterId: id } }).catch((error) =>
      console.error('Navigation failed:', error),
    )

  // Route polyline rendering is restored once the backend routing service ships;
  // until then the preview only shows the plan's clusters for context.
  useClusterBoundaryLayer({ onBoundaryClick: navToCluster })
  useClusterMarkerLayer({ onClusterClick: navToCluster })

  useEffect(() => {
    if (!isMapAlive(map)) return
    const ids = new Set(clusterIds)
    const points = markers.data.filter((c) => ids.has(c.id))
    if (points.length === 0) return
    const lngs = points.map((p) => p.longitude)
    const lats = points.map((p) => p.latitude)
    map.fitBounds(
      [
        [Math.min(...lngs), Math.min(...lats)],
        [Math.max(...lngs), Math.max(...lats)],
      ],
      { padding: 64, maxZoom: 16 },
    )
  }, [map, markers, clusterIds])

  return null
}

const WateringPlanPreviewRoute = ({ wateringPlan }: WateringPlanPreviewRouteProps) => {
  const [centerLat, centerLng] = useStore.getState().mapCenter
  const clusterIds = wateringPlan.treeclusters.map((tc) => tc.id)

  return (
    <MapPreview
      center={[centerLng, centerLat]}
      zoom={13}
      interactive
      className="h-[40rem]"
      ariaLabel="Karte mit den Bewässerungsgruppen des Plans"
    >
      <Suspense fallback={<Loading className="justify-center" label="Lade Karte..." />}>
        <RoutePreviewLayers clusterIds={clusterIds} />
      </Suspense>
    </MapPreview>
  )
}

export default WateringPlanPreviewRoute
