import { useEffect } from 'react'
import { useSuspenseQuery } from '@tanstack/react-query'
import { useNavigate } from '@tanstack/react-router'
import type { GeoJSONSource, LngLatBoundsLike, MapLayerMouseEvent } from 'maplibre-gl'
import type { PlanResponse } from '@green-ecolution/backend-client'
import { wateringPlanQueries } from '@/api/queries'
import { useMaplibreMap } from '@/components/map-gl/MapContext'
import { usePointerCursor } from '@/components/map-gl/layers/usePointerCursor'
import { isMapAlive } from '@/components/map-gl/mapReady'
import { setSourceData } from '@/components/map-gl/setSourceData'
import { wasteBinQueries } from '../../api/queries'
import type { WasteBin } from '../../api/client'
import { binsToGeoJson, FILL_STATE_COLOR_EXPRESSION } from '../../lib/mapStyle'

const SOURCE = 'waste-route-bins'
const LAYER = 'waste-route-bins'

const binBounds = (bins: WasteBin[]): LngLatBoundsLike | undefined => {
  if (bins.length === 0) return undefined
  const lngs = bins.map(({ longitude }) => longitude)
  const lats = bins.map(({ latitude }) => latitude)
  return [
    [Math.min(...lngs), Math.min(...lats)],
    [Math.max(...lngs), Math.max(...lats)],
  ]
}

/** Draws the plan's bins above the core's route; a plan without a route is framed on its bins. */
const BinRouteOverlay = ({ plan }: { plan: PlanResponse }) => {
  const map = useMaplibreMap()
  const navigate = useNavigate()
  const { data: bins } = useSuspenseQuery(wasteBinQueries.byIds(plan.stops))
  const { data: route } = useSuspenseQuery(wateringPlanQueries.route(plan.id))
  const hasRoute = Boolean(route?.geometry.coordinates.length)

  useEffect(() => {
    if (!map.getSource(SOURCE)) {
      map.addSource(SOURCE, { type: 'geojson', data: { type: 'FeatureCollection', features: [] } })
    }
    if (!map.getLayer(LAYER)) {
      map.addLayer({
        id: LAYER,
        type: 'circle',
        source: SOURCE,
        paint: {
          'circle-radius': 9,
          'circle-color': FILL_STATE_COLOR_EXPRESSION,
          'circle-stroke-width': 2,
          'circle-stroke-color': '#ffffff',
        },
      })
    }
    return () => {
      if (!isMapAlive(map)) return
      if (map.getLayer(LAYER)) map.removeLayer(LAYER)
      if (map.getSource(SOURCE)) map.removeSource(SOURCE)
    }
  }, [map])

  useEffect(() => {
    if (!isMapAlive(map)) return
    setSourceData(map.getSource<GeoJSONSource>(SOURCE), binsToGeoJson(bins))
  }, [map, bins])

  useEffect(() => {
    if (hasRoute || !isMapAlive(map)) return
    const bounds = binBounds(bins)
    if (bounds) map.fitBounds(bounds, { padding: 64, maxZoom: 16 })
  }, [map, hasRoute, bins])

  usePointerCursor(LAYER)

  useEffect(() => {
    const openBin = (e: MapLayerMouseEvent) => {
      const id = e.features?.[0]?.properties?.id as string | undefined
      if (!id) return
      navigate({ to: '/waste-bins/$wasteBinId', params: { wasteBinId: id } }).catch((error) =>
        console.error('Navigation failed:', error),
      )
    }
    map.on('click', LAYER, openBin)
    return () => {
      map.off('click', LAYER, openBin)
    }
  }, [map, navigate])

  return null
}

export default BinRouteOverlay
