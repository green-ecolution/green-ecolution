import { useEffect, useMemo, Suspense } from 'react'
import { useSuspenseQuery } from '@tanstack/react-query'
import { Loading } from '@green-ecolution/ui'
import { useTranslation } from 'react-i18next'
import type { PlanResponse } from '@green-ecolution/backend-client'
import { routingStartPointsQuery, wateringPlanQueries } from '@/api/queries'
import RoutePointMarkers, {
  buildRoutePoints,
  type RoutePointMarkerData,
} from '@/components/map-gl/RoutePointMarkers'
import { useMapCenter } from '@/components/map-gl/useMapView'
import MapPreview from '@/components/map-gl/MapPreview'
import { useMaplibreMap } from '@/components/map-gl/MapContext'
import useRouteLayer from '@/components/map-gl/layers/useRouteLayer'
import { isMapAlive } from '@/components/map-gl/mapReady'
import type { ModulePlanning } from '@/modules/types'

interface WateringPlanPreviewRouteProps {
  wateringPlan: PlanResponse
  planning: ModulePlanning | undefined
}

const RoutePreviewLayers = ({
  planId,
  startPointName,
}: {
  planId: string
  startPointName?: string | null
}) => {
  const map = useMaplibreMap()
  const { data: route } = useSuspenseQuery(wateringPlanQueries.route(planId))
  const { data: startPoints } = useSuspenseQuery(routingStartPointsQuery())
  const routePoints = useMemo<RoutePointMarkerData[]>(
    () => buildRoutePoints(startPoints, startPointName, route?.stationVisits),
    [startPoints, startPointName, route],
  )

  const routeCoordinates = route?.geometry.coordinates as [number, number][] | undefined

  useRouteLayer({ coordinates: routeCoordinates })

  useEffect(() => {
    if (!isMapAlive(map) || !routeCoordinates?.length) return
    let [minLng, minLat] = routeCoordinates[0]
    let [maxLng, maxLat] = routeCoordinates[0]
    for (const [lng, lat] of routeCoordinates) {
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
  }, [map, routeCoordinates])

  return <RoutePointMarkers points={routePoints} />
}

/** The core draws the route and its stations; the plan module draws its stops on top. */
const WateringPlanPreviewRoute = ({ wateringPlan, planning }: WateringPlanPreviewRouteProps) => {
  const { t } = useTranslation('wateringPlan')
  const [centerLat, centerLng] = useMapCenter()

  return (
    <MapPreview
      center={[centerLng, centerLat]}
      zoom={13}
      interactive
      className="h-[40rem]"
      ariaLabel={t('detail.routePreviewAriaLabel')}
    >
      <Suspense
        fallback={
          <Loading className="justify-center" label={t('detail.routePreviewLoadingLabel')} />
        }
      >
        <RoutePreviewLayers planId={wateringPlan.id} startPointName={wateringPlan.startPointName} />
        {planning && <planning.RouteOverlay plan={wateringPlan} />}
      </Suspense>
    </MapPreview>
  )
}

export default WateringPlanPreviewRoute
