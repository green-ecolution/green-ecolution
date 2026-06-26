import { useEffect } from 'react'
import { useSuspenseQuery } from '@tanstack/react-query'
import type { GeoJSONSource, MapLayerMouseEvent } from 'maplibre-gl'
import type { FeatureCollection, Point } from 'geojson'
import type { WateringStatus } from '@green-ecolution/backend-client'
import { clusterMarkersQuery } from '@/api/queries'
import { useMaplibreMap } from '../MapContext'
import { LAYERS, SOURCES, STATUS_COLOR_EXPRESSION, TREE_ZOOM_THRESHOLD } from '../mapStyle'
import { isMapAlive } from '../mapReady'

export interface UseClusterMarkerLayerOptions {
  onClusterClick?: (clusterId: string) => void
  wateringStatuses?: WateringStatus[]
  nameFilter?: string
  interactive?: boolean
}

const useClusterMarkerLayer = ({
  onClusterClick,
  wateringStatuses,
  nameFilter,
  interactive = true,
}: UseClusterMarkerLayerOptions = {}) => {
  const map = useMaplibreMap()
  const { data } = useSuspenseQuery(clusterMarkersQuery())

  useEffect(() => {
    if (!map.getSource(SOURCES.clusterMarkers)) {
      map.addSource(SOURCES.clusterMarkers, {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: [] },
      })
    }
    if (!map.getLayer(LAYERS.clusterMarkers)) {
      map.addLayer({
        id: LAYERS.clusterMarkers,
        type: 'circle',
        source: SOURCES.clusterMarkers,
        maxzoom: TREE_ZOOM_THRESHOLD,
        paint: {
          'circle-radius': ['step', ['get', 'count'], 16, 25, 20, 100, 26],
          'circle-color': STATUS_COLOR_EXPRESSION,
          'circle-stroke-width': 2,
          'circle-stroke-color': '#ffffff',
        },
      })
    }
    if (!map.getLayer(LAYERS.clusterMarkerCount)) {
      map.addLayer({
        id: LAYERS.clusterMarkerCount,
        type: 'symbol',
        source: SOURCES.clusterMarkers,
        maxzoom: TREE_ZOOM_THRESHOLD,
        layout: {
          'text-field': ['to-string', ['get', 'count']],
          'text-font': ['Noto Sans Bold'],
          'text-size': 13,
        },
        paint: {
          'text-color': '#ffffff',
          'text-halo-color': 'rgba(0, 0, 0, 0.35)',
          'text-halo-width': 1,
        },
      })
    }
    return () => {
      if (!isMapAlive(map)) return
      for (const id of [LAYERS.clusterMarkerCount, LAYERS.clusterMarkers]) {
        if (map.getLayer(id)) map.removeLayer(id)
      }
      if (map.getSource(SOURCES.clusterMarkers)) map.removeSource(SOURCES.clusterMarkers)
    }
  }, [map])

  useEffect(() => {
    if (!isMapAlive(map)) return
    const statusSet = wateringStatuses?.length ? new Set(wateringStatuses) : null
    const term = nameFilter?.trim().toLowerCase() ?? ''
    const fc: FeatureCollection<Point> = {
      type: 'FeatureCollection',
      features: data.data
        .filter(
          (c) =>
            (!statusSet || statusSet.has(c.wateringStatus)) &&
            (!term || c.name.toLowerCase().includes(term)),
        )
        .map((c) => ({
          type: 'Feature',
          geometry: { type: 'Point', coordinates: [c.longitude, c.latitude] },
          properties: { id: c.id, name: c.name, count: c.treeCount, status: c.wateringStatus },
        })),
    }
    map.getSource<GeoJSONSource>(SOURCES.clusterMarkers)?.setData(fc)
  }, [map, data, wateringStatuses, nameFilter])

  useEffect(() => {
    if (!interactive) return
    const onClick = (e: MapLayerMouseEvent) => {
      const feature = e.features?.[0]
      if (!feature) return
      const geometry = feature.geometry as Point
      map.flyTo({
        center: [geometry.coordinates[0], geometry.coordinates[1]],
        zoom: TREE_ZOOM_THRESHOLD + 1,
      })
      onClusterClick?.(feature.properties?.id as string)
    }
    const enter = () => {
      map.getCanvas().style.cursor = 'pointer'
    }
    const leave = () => {
      map.getCanvas().style.cursor = ''
    }
    map.on('click', LAYERS.clusterMarkers, onClick)
    map.on('mouseenter', LAYERS.clusterMarkers, enter)
    map.on('mouseleave', LAYERS.clusterMarkers, leave)
    return () => {
      map.off('click', LAYERS.clusterMarkers, onClick)
      map.off('mouseenter', LAYERS.clusterMarkers, enter)
      map.off('mouseleave', LAYERS.clusterMarkers, leave)
    }
  }, [map, onClusterClick, interactive])
}

export default useClusterMarkerLayer
