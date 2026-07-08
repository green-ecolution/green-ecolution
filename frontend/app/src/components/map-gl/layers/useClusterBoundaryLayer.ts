import { useEffect } from 'react'
import { useSuspenseQuery } from '@tanstack/react-query'
import type { GeoJSONSource, MapLayerMouseEvent } from 'maplibre-gl'
import type { FeatureCollection, Polygon } from 'geojson'
import type { WateringStatus } from '@green-ecolution/backend-client'
import { clusterBoundariesQuery } from '@/api/queries'
import { useMaplibreMap } from '../MapContext'
import { LAYERS, SOURCES, STATUS_COLOR_EXPRESSION } from '../mapStyle'
import { isMapAlive } from '../mapReady'
import { usePointerCursor } from './usePointerCursor'

export interface UseClusterBoundaryLayerOptions {
  onBoundaryClick?: (clusterId: string) => void
  selectedClusterId?: string
  wateringStatuses?: WateringStatus[]
  nameFilter?: string
  interactive?: boolean
  clusterIds?: string[]
}

const useClusterBoundaryLayer = ({
  onBoundaryClick,
  selectedClusterId,
  wateringStatuses,
  nameFilter,
  interactive = true,
  clusterIds,
}: UseClusterBoundaryLayerOptions = {}) => {
  const map = useMaplibreMap()
  const { data } = useSuspenseQuery(clusterBoundariesQuery())

  useEffect(() => {
    if (!map.getSource(SOURCES.clusterBoundaries)) {
      map.addSource(SOURCES.clusterBoundaries, {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: [] },
      })
    }
    if (!map.getLayer(LAYERS.boundaryFill)) {
      map.addLayer({
        id: LAYERS.boundaryFill,
        type: 'fill',
        source: SOURCES.clusterBoundaries,
        paint: { 'fill-color': STATUS_COLOR_EXPRESSION, 'fill-opacity': 0.15 },
      })
    }
    if (!map.getLayer(LAYERS.boundaryLine)) {
      map.addLayer({
        id: LAYERS.boundaryLine,
        type: 'line',
        source: SOURCES.clusterBoundaries,
        paint: { 'line-color': STATUS_COLOR_EXPRESSION, 'line-width': 2 },
        layout: { 'line-join': 'round', 'line-cap': 'round' },
      })
    }
    return () => {
      if (!isMapAlive(map)) return
      for (const id of [LAYERS.boundaryLine, LAYERS.boundaryFill]) {
        if (map.getLayer(id)) map.removeLayer(id)
      }
      if (map.getSource(SOURCES.clusterBoundaries)) map.removeSource(SOURCES.clusterBoundaries)
    }
  }, [map])

  useEffect(() => {
    if (!isMapAlive(map)) return
    const statusSet = wateringStatuses?.length ? new Set(wateringStatuses) : null
    const idSet = clusterIds?.length ? new Set(clusterIds) : null
    const term = nameFilter?.trim().toLowerCase() ?? ''
    const fc: FeatureCollection<Polygon> = {
      type: 'FeatureCollection',
      features: data.data
        .filter(
          (b) =>
            (!statusSet || statusSet.has(b.wateringStatus)) &&
            (!idSet || idSet.has(b.id)) &&
            (!term || b.name.toLowerCase().includes(term)),
        )
        .map((b) => ({
          type: 'Feature',
          geometry: b.boundary as Polygon,
          properties: { id: b.id, name: b.name, status: b.wateringStatus },
        })),
    }
    map.getSource<GeoJSONSource>(SOURCES.clusterBoundaries)?.setData(fc)
  }, [map, data, wateringStatuses, clusterIds, nameFilter])

  useEffect(() => {
    if (
      !isMapAlive(map) ||
      !map.getLayer(LAYERS.boundaryFill) ||
      !map.getLayer(LAYERS.boundaryLine)
    )
      return
    const selected = selectedClusterId ?? '__none__'
    map.setPaintProperty(LAYERS.boundaryFill, 'fill-opacity', [
      'case',
      ['==', ['get', 'id'], selected],
      0.35,
      0.1,
    ])
    map.setPaintProperty(LAYERS.boundaryLine, 'line-width', [
      'case',
      ['==', ['get', 'id'], selected],
      4,
      1.5,
    ])
  }, [map, selectedClusterId])

  usePointerCursor(LAYERS.boundaryFill, interactive)

  useEffect(() => {
    if (!interactive) return
    const onClick = (e: MapLayerMouseEvent) => {
      const feature = e.features?.[0]
      if (!feature) return
      onBoundaryClick?.(feature.properties?.id as string)
    }
    map.on('click', LAYERS.boundaryFill, onClick)
    return () => {
      map.off('click', LAYERS.boundaryFill, onClick)
    }
  }, [map, onBoundaryClick, interactive])
}

export default useClusterBoundaryLayer
