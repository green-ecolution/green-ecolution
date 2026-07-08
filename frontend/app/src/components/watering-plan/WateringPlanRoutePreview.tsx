import { Suspense, useCallback, useEffect } from 'react'
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

interface WateringPlanPreviewRouteProps {
  wateringPlan: WateringPlan
}

const RoutePreviewLayers = ({ planId, clusterIds }: { planId: string; clusterIds: string[] }) => {
  const map = useMaplibreMap()
  const navigate = useNavigate()
  const { data: markers } = useSuspenseQuery(clusterMarkersQuery())
  const { data: route } = useSuspenseQuery(wateringPlanRouteQuery(planId))

  const navToCluster = useCallback(
    (id: string) =>
      navigate({ to: '/treecluster/$treeclusterId', params: { treeclusterId: id } }).catch(
        (error) => console.error('Navigation failed:', error),
      ),
    [navigate],
  )

  const routeCoordinates = route?.geometry.coordinates as [number, number][] | undefined

  useClusterBoundaryLayer({ onBoundaryClick: navToCluster })
  useRouteLayer({ coordinates: routeCoordinates })
  // flyToOnClick off: the click navigates away, so animating the unmounting map is wasted.
  useClusterMarkerLayer({ onClusterClick: navToCluster, flyToOnClick: false })

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
    const lngs = points.map((p) => p[0])
    const lats = points.map((p) => p[1])
    map.fitBounds(
      [
        [Math.min(...lngs), Math.min(...lats)],
        [Math.max(...lngs), Math.max(...lats)],
      ],
      { padding: 64, maxZoom: 16 },
    )
  }, [map, markers, clusterIds, routeCoordinates])

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
      ariaLabel="Karte mit der Route und den Bewässerungsgruppen des Plans"
    >
      <Suspense fallback={<Loading className="justify-center" label="Lade Karte..." />}>
        <RoutePreviewLayers planId={wateringPlan.id} clusterIds={clusterIds} />
      </Suspense>
    </MapPreview>
  )
}

export default WateringPlanPreviewRoute
