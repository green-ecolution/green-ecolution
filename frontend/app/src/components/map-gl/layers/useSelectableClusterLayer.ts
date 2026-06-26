import { useEffect, useMemo } from 'react'
import { useSuspenseQuery } from '@tanstack/react-query'
import type { GeoJSONSource, MapLayerMouseEvent } from 'maplibre-gl'
import type { FeatureCollection, Point } from 'geojson'
import { clusterMarkersQuery } from '@/api/queries'
import { useMaplibreMap } from '../MapContext'
import { LAYERS, SOURCES, STATUS_COLOR_EXPRESSION } from '../mapStyle'
import { isMapAlive } from '../mapReady'

export interface UseSelectableClusterLayerOptions {
  selectedIds: string[]
  disabledIds?: string[]
  onToggle: (clusterId: string) => void
}

const useSelectableClusterLayer = ({
  selectedIds,
  disabledIds,
  onToggle,
}: UseSelectableClusterLayerOptions) => {
  const map = useMaplibreMap()
  const { data } = useSuspenseQuery(clusterMarkersQuery())
  const selectedSet = useMemo(() => new Set(selectedIds), [selectedIds])
  const disabledSet = useMemo(() => new Set(disabledIds ?? []), [disabledIds])

  useEffect(() => {
    if (!map.getSource(SOURCES.selectClusters)) {
      map.addSource(SOURCES.selectClusters, {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: [] },
      })
    }
    if (!map.getLayer(LAYERS.selectClusterPoints)) {
      map.addLayer({
        id: LAYERS.selectClusterPoints,
        type: 'circle',
        source: SOURCES.selectClusters,
        paint: {
          'circle-radius': ['step', ['get', 'count'], 15, 25, 18, 100, 22],
          'circle-color': [
            'case',
            ['boolean', ['get', 'disabled'], false],
            '#A2A2A2',
            STATUS_COLOR_EXPRESSION,
          ],
          'circle-opacity': ['case', ['boolean', ['get', 'disabled'], false], 0.35, 1],
          'circle-stroke-width': ['case', ['boolean', ['get', 'selected'], false], 4, 2],
          'circle-stroke-color': [
            'case',
            ['boolean', ['get', 'selected'], false],
            '#1f2937',
            '#ffffff',
          ],
        },
      })
    }
    if (!map.getLayer(LAYERS.selectClusterCount)) {
      map.addLayer({
        id: LAYERS.selectClusterCount,
        type: 'symbol',
        source: SOURCES.selectClusters,
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
      for (const id of [LAYERS.selectClusterCount, LAYERS.selectClusterPoints]) {
        if (map.getLayer(id)) map.removeLayer(id)
      }
      if (map.getSource(SOURCES.selectClusters)) map.removeSource(SOURCES.selectClusters)
    }
  }, [map])

  useEffect(() => {
    if (!isMapAlive(map)) return
    const fc: FeatureCollection<Point> = {
      type: 'FeatureCollection',
      features: data.data.map((c) => ({
        type: 'Feature',
        geometry: { type: 'Point', coordinates: [c.longitude, c.latitude] },
        properties: {
          id: c.id,
          name: c.name,
          count: c.treeCount,
          status: c.wateringStatus,
          selected: selectedSet.has(c.id),
          disabled: disabledSet.has(c.id),
        },
      })),
    }
    map.getSource<GeoJSONSource>(SOURCES.selectClusters)?.setData(fc)
  }, [map, data, selectedSet, disabledSet])

  useEffect(() => {
    const onClick = (e: MapLayerMouseEvent) => {
      const feature = e.features?.[0]
      if (!feature) return
      if (feature.properties?.disabled) return
      onToggle(feature.properties?.id as string)
    }
    const enter = () => {
      map.getCanvas().style.cursor = 'pointer'
    }
    const leave = () => {
      map.getCanvas().style.cursor = ''
    }
    map.on('click', LAYERS.selectClusterPoints, onClick)
    map.on('mouseenter', LAYERS.selectClusterPoints, enter)
    map.on('mouseleave', LAYERS.selectClusterPoints, leave)
    return () => {
      map.off('click', LAYERS.selectClusterPoints, onClick)
      map.off('mouseenter', LAYERS.selectClusterPoints, enter)
      map.off('mouseleave', LAYERS.selectClusterPoints, leave)
    }
  }, [map, onToggle])
}

export default useSelectableClusterLayer
