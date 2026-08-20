import { useEffect, useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import type { ExpressionSpecification, GeoJSONSource, MapLayerMouseEvent } from 'maplibre-gl'
import type { FeatureCollection, Point } from 'geojson'
import type { TreeMarkerResponse } from '@green-ecolution/backend-client'
import { treeQueries } from '@/api/queries'
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

// Opacity of trees that belong to another organization.
const UNSELECTABLE_DIM: ExpressionSpecification = [
  'case',
  ['boolean', ['get', 'selectable'], true],
  1,
  0.3,
]

export interface ForeignTree {
  id: string
  organizationId: string
}

export interface UseSelectableTreeLayerOptions {
  selectedIds: string[]
  onToggle: (treeId: string) => void
  /** Trees of other organizations stay visible but unselectable, because a
   *  cluster must not mix organizations. Undefined makes everything
   *  selectable. */
  organizationId?: string
  /** Called instead of `onToggle` when a tree of another organization is
   *  clicked. Without it such a click does nothing. */
  onForeignTree?: (tree: ForeignTree) => void
}

export const toFC = (
  trees: TreeMarkerResponse[],
  selected: Set<string>,
  organizationId?: string,
): FeatureCollection<Point> => ({
  type: 'FeatureCollection',
  features: trees.map((t) => ({
    type: 'Feature',
    geometry: { type: 'Point', coordinates: [t.longitude, t.latitude] },
    properties: {
      id: t.id,
      status: t.wateringStatus,
      selected: selected.has(t.id),
      organizationId: t.organizationId,
      selectable: !organizationId || t.organizationId === organizationId,
    },
  })),
})

const useSelectableTreeLayer = ({
  selectedIds,
  onToggle,
  organizationId,
  onForeignTree,
}: UseSelectableTreeLayerOptions) => {
  const map = useMaplibreMap()
  const bbox = useViewportBBox()
  // Deliberately unfiltered: foreign trees are drawn as dimmed context so the
  // map does not look empty when the chosen organization owns nothing here.
  const { data } = useQuery(treeQueries.markers({ bbox }))
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
          // Foreign trees stay on the map as context, but visibly out of reach.
          'circle-opacity': UNSELECTABLE_DIM,
          'circle-stroke-opacity': UNSELECTABLE_DIM,
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
          filter: [
            'all',
            ['!', ['boolean', ['get', 'selected'], false]],
            ['boolean', ['get', 'selectable'], true],
          ],
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
    map
      .getSource<GeoJSONSource>(SOURCES.selectTrees)
      ?.setData(toFC(data?.data ?? [], selectedSet, organizationId))
  }, [map, data, selectedSet, organizationId])

  usePointerCursor(LAYERS.selectTreePoints)

  useEffect(() => {
    const onClick = (e: MapLayerMouseEvent) => {
      const properties = e.features?.[0]?.properties
      if (!properties) return
      const id = properties.id as string
      if (properties.selectable === false) {
        onForeignTree?.({ id, organizationId: properties.organizationId as string })
        return
      }
      onToggle(id)
    }
    map.on('click', LAYERS.selectTreePoints, onClick)
    return () => {
      map.off('click', LAYERS.selectTreePoints, onClick)
    }
  }, [map, onToggle, onForeignTree])
}

export default useSelectableTreeLayer
