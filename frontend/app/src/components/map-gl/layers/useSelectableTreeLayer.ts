import { useEffect, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import type { GeoJSONSource, MapLayerMouseEvent } from 'maplibre-gl'
import type { FeatureCollection, Point } from 'geojson'
import type { TreeMarkerResponse } from '@green-ecolution/backend-client'
import { treeMarkersQuery } from '@/api/queries'
import { useMaplibreMap } from '../MapContext'
import {
  CHECK_ICON_IMAGE,
  CHECK_ICON_URL,
  LAYERS,
  SOURCES,
  STATUS_COLOR_EXPRESSION,
  TREE_ICON_IMAGE,
  TREE_ICON_URL,
} from '../mapStyle'
import useViewportBBox from '../hooks/useViewportBBox'
import { isMapAlive } from '../mapReady'
import { usePointerCursor } from './usePointerCursor'

export interface UseSelectableTreeLayerOptions {
  selectedIds: string[]
  onToggle: (treeId: string) => void
}

const toFC = (trees: TreeMarkerResponse[], selected: Set<string>): FeatureCollection<Point> => ({
  type: 'FeatureCollection',
  features: trees.map((t) => ({
    type: 'Feature',
    geometry: { type: 'Point', coordinates: [t.longitude, t.latitude] },
    properties: { id: t.id, status: t.wateringStatus, selected: selected.has(t.id) },
  })),
})

const useSelectableTreeLayer = ({ selectedIds, onToggle }: UseSelectableTreeLayerOptions) => {
  const map = useMaplibreMap()
  const bbox = useViewportBBox()
  const { data } = useQuery(treeMarkersQuery({ bbox }))
  const selectedSet = useMemo(() => new Set(selectedIds), [selectedIds])

  useEffect(() => {
    let cancelled = false

    if (!map.getSource(SOURCES.selectTrees)) {
      map.addSource(SOURCES.selectTrees, {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: [] },
      })
    }
    if (!map.getLayer(LAYERS.selectTreePoints)) {
      map.addLayer({
        id: LAYERS.selectTreePoints,
        type: 'circle',
        source: SOURCES.selectTrees,
        paint: {
          'circle-radius': ['interpolate', ['linear'], ['zoom'], 13, 8, 17, 11, 22, 14],
          'circle-color': STATUS_COLOR_EXPRESSION,
          'circle-stroke-width': 1.5,
          'circle-stroke-color': '#ffffff',
        },
      })
    }

    const ensureIconLayers = () => {
      if (cancelled) return
      // Unselected trees keep the leaf icon, but collision-managed so dense areas
      // declutter instead of overlapping; more icons appear as you zoom in.
      if (map.hasImage(TREE_ICON_IMAGE) && !map.getLayer(LAYERS.selectTreeIcon)) {
        map.addLayer({
          id: LAYERS.selectTreeIcon,
          type: 'symbol',
          source: SOURCES.selectTrees,
          filter: ['!', ['boolean', ['get', 'selected'], false]],
          layout: {
            'icon-image': TREE_ICON_IMAGE,
            'icon-size': ['interpolate', ['linear'], ['zoom'], 13, 0.38, 17, 0.52, 22, 0.7],
            'icon-allow-overlap': true,
            'icon-ignore-placement': true,
          },
        })
      }
      // Selected trees show the check, always (selected ones are few).
      if (map.hasImage(CHECK_ICON_IMAGE) && !map.getLayer(LAYERS.selectTreeCheck)) {
        map.addLayer({
          id: LAYERS.selectTreeCheck,
          type: 'symbol',
          source: SOURCES.selectTrees,
          filter: ['boolean', ['get', 'selected'], false],
          layout: {
            'icon-image': CHECK_ICON_IMAGE,
            'icon-size': ['interpolate', ['linear'], ['zoom'], 13, 0.5, 17, 0.7, 22, 0.9],
            'icon-allow-overlap': true,
            'icon-ignore-placement': true,
          },
        })
      }
    }

    const loadImage = (id: string, url: string) => {
      if (map.hasImage(id)) {
        ensureIconLayers()
        return
      }
      const img = new Image(48, 48)
      img.onload = () => {
        if (cancelled) return
        if (!map.hasImage(id)) map.addImage(id, img, { pixelRatio: 2 })
        ensureIconLayers()
      }
      img.src = url
    }

    loadImage(TREE_ICON_IMAGE, TREE_ICON_URL)
    loadImage(CHECK_ICON_IMAGE, CHECK_ICON_URL)

    return () => {
      cancelled = true
      if (!isMapAlive(map)) return
      for (const id of [LAYERS.selectTreeCheck, LAYERS.selectTreeIcon, LAYERS.selectTreePoints]) {
        if (map.getLayer(id)) map.removeLayer(id)
      }
      if (map.getSource(SOURCES.selectTrees)) map.removeSource(SOURCES.selectTrees)
    }
  }, [map])

  useEffect(() => {
    if (!isMapAlive(map)) return
    map.getSource<GeoJSONSource>(SOURCES.selectTrees)?.setData(toFC(data?.data ?? [], selectedSet))
  }, [map, data, selectedSet])

  usePointerCursor(LAYERS.selectTreePoints)

  useEffect(() => {
    const onClick = (e: MapLayerMouseEvent) => {
      const feature = e.features?.[0]
      if (!feature) return
      onToggle(feature.properties?.id as string)
    }
    map.on('click', LAYERS.selectTreePoints, onClick)
    return () => {
      map.off('click', LAYERS.selectTreePoints, onClick)
    }
  }, [map, onToggle])
}

export default useSelectableTreeLayer
