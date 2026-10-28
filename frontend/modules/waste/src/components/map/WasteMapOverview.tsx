import { useCallback, useEffect } from 'react'
import { useNavigate } from '@tanstack/react-router'
import { useSuspenseQuery } from '@tanstack/react-query'
import type { GeoJSONSource, MapLayerMouseEvent } from 'maplibre-gl'
import MapBackgroundClick from '@/components/map-gl/MapBackgroundClick'
import { useMaplibreMap } from '@/components/map-gl/MapContext'
import { usePointerCursor } from '@/components/map-gl/layers/usePointerCursor'
import { isMapAlive } from '@/components/map-gl/mapReady'
import { setSourceData } from '@/components/map-gl/setSourceData'
import { wasteBinQueries } from '../../api/queries'
import { useWasteMapSearch } from '../../lib/mapSearch'
import { binsToGeoJson, FILL_STATE_COLOR_EXPRESSION, LAYERS, SOURCES } from '../../lib/mapStyle'

const WasteMapOverview = () => {
  const map = useMaplibreMap()
  const navigate = useNavigate({ from: '/map' })
  const { wasteBin } = useWasteMapSearch()
  const { data: bins } = useSuspenseQuery(wasteBinQueries.markers())

  useEffect(() => {
    if (!map.getSource(SOURCES.bins)) {
      map.addSource(SOURCES.bins, {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: [] },
      })
    }
    if (!map.getLayer(LAYERS.bins)) {
      map.addLayer({
        id: LAYERS.bins,
        type: 'circle',
        source: SOURCES.bins,
        paint: {
          'circle-radius': 9,
          'circle-color': FILL_STATE_COLOR_EXPRESSION,
          'circle-stroke-width': 2,
          'circle-stroke-color': '#ffffff',
        },
      })
    }
    if (!map.getLayer(LAYERS.selected)) {
      map.addLayer({
        id: LAYERS.selected,
        type: 'circle',
        source: SOURCES.bins,
        filter: ['==', ['get', 'id'], ''],
        paint: {
          'circle-radius': 14,
          'circle-color': 'rgba(0, 0, 0, 0)',
          'circle-stroke-width': 3,
          'circle-stroke-color': '#ffffff',
        },
      })
    }
    return () => {
      if (!isMapAlive(map)) return
      for (const id of [LAYERS.selected, LAYERS.bins]) {
        if (map.getLayer(id)) map.removeLayer(id)
      }
      if (map.getSource(SOURCES.bins)) map.removeSource(SOURCES.bins)
    }
  }, [map])

  useEffect(() => {
    if (!isMapAlive(map)) return
    setSourceData(map.getSource<GeoJSONSource>(SOURCES.bins), binsToGeoJson(bins))
  }, [map, bins])

  useEffect(() => {
    if (!isMapAlive(map) || !map.getLayer(LAYERS.selected)) return
    map.setFilter(LAYERS.selected, ['==', ['get', 'id'], wasteBin ?? ''])
  }, [map, wasteBin])

  usePointerCursor(LAYERS.bins)

  const selectBin = useCallback(
    (id: string | undefined) => {
      navigate({ to: '/map', search: (prev) => ({ ...prev, wasteBin: id }) }).catch((error) =>
        console.error('Navigation failed:', error),
      )
    },
    [navigate],
  )

  useEffect(() => {
    const onClick = (e: MapLayerMouseEvent) => {
      const id = e.features?.[0]?.properties?.id as string | undefined
      if (id) selectBin(id)
    }
    map.on('click', LAYERS.bins, onClick)
    return () => {
      map.off('click', LAYERS.bins, onClick)
    }
  }, [map, selectBin])

  const closePopup = useCallback(() => {
    if (wasteBin) selectBin(undefined)
  }, [wasteBin, selectBin])

  return <MapBackgroundClick onBackground={closePopup} />
}

export default WasteMapOverview
