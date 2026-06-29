import { useEffect, useMemo } from 'react'
import type { ExpressionSpecification, GeoJSONSource, MapLayerMouseEvent } from 'maplibre-gl'
import type { FeatureCollection, Point } from 'geojson'
import type { WateringStatus } from '@green-ecolution/backend-client'
import { useMaplibreMap } from '../MapContext'
import { STATUS_COLOR_EXPRESSION, TREE_ICON_IMAGE, TREE_ICON_URL } from '../mapStyle'
import { isMapAlive } from '../mapReady'
import { usePointerCursor } from './usePointerCursor'

export interface TreeMarkerPoint {
  id: string
  longitude: number
  latitude: number
  status: WateringStatus
  // Dimmed and non-selectable (e.g. a tree that already has a sensor).
  disabled?: boolean
  selected?: boolean
}

export interface UseTreeMarkerLayerOptions {
  trees: TreeMarkerPoint[]
  sourceId: string
  circleLayerId: string
  iconLayerId: string
  minZoom?: number
  circleRadius?: ExpressionSpecification
  iconSize?: ExpressionSpecification
  onTreeClick?: (treeId: string) => void
  interactive?: boolean
}

const DEFAULT_CIRCLE_RADIUS: ExpressionSpecification = [
  'interpolate',
  ['linear'],
  ['zoom'],
  17,
  9,
  19,
  14,
  22,
  18,
]

const DEFAULT_ICON_SIZE: ExpressionSpecification = [
  'interpolate',
  ['linear'],
  ['zoom'],
  17,
  0.46,
  19,
  0.72,
  22,
  0.95,
]

const toFC = (trees: TreeMarkerPoint[]): FeatureCollection<Point> => ({
  type: 'FeatureCollection',
  features: trees.map((t) => ({
    type: 'Feature',
    geometry: { type: 'Point', coordinates: [t.longitude, t.latitude] },
    properties: {
      id: t.id,
      status: t.status,
      selected: t.selected ?? false,
      disabled: t.disabled ?? false,
    },
  })),
})

// Renders trees as a colored watering-status circle topped by the shared white
// tree icon. Both the main map (`useTreeLayers`) and the sensor assign dialog
// drive it; the `selected`/`disabled` feature flags default to false, so callers
// that don't use selection get the plain main-map styling.
const useTreeMarkerLayer = ({
  trees,
  sourceId,
  circleLayerId,
  iconLayerId,
  minZoom,
  circleRadius = DEFAULT_CIRCLE_RADIUS,
  iconSize = DEFAULT_ICON_SIZE,
  onTreeClick,
  interactive = true,
}: UseTreeMarkerLayerOptions) => {
  const map = useMaplibreMap()
  const fc = useMemo(() => toFC(trees), [trees])

  useEffect(() => {
    let cancelled = false

    if (!map.getSource(sourceId)) {
      map.addSource(sourceId, {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: [] },
      })
    }
    if (!map.getLayer(circleLayerId)) {
      map.addLayer({
        id: circleLayerId,
        type: 'circle',
        source: sourceId,
        ...(minZoom != null ? { minzoom: minZoom } : {}),
        paint: {
          'circle-radius': circleRadius,
          'circle-color': STATUS_COLOR_EXPRESSION,
          'circle-opacity': ['case', ['boolean', ['get', 'disabled'], false], 0.4, 1],
          'circle-stroke-width': ['case', ['boolean', ['get', 'selected'], false], 4, 2],
          'circle-stroke-color': [
            'case',
            ['boolean', ['get', 'selected'], false],
            '#486725',
            '#ffffff',
          ],
        },
      })
    }

    const addIconLayer = () => {
      if (cancelled || map.getLayer(iconLayerId)) return
      map.addLayer({
        id: iconLayerId,
        type: 'symbol',
        source: sourceId,
        ...(minZoom != null ? { minzoom: minZoom } : {}),
        layout: {
          'icon-image': TREE_ICON_IMAGE,
          'icon-size': iconSize,
          'icon-allow-overlap': true,
          'icon-ignore-placement': true,
        },
        paint: { 'icon-opacity': ['case', ['boolean', ['get', 'disabled'], false], 0.4, 1] },
      })
    }

    if (map.hasImage(TREE_ICON_IMAGE)) {
      addIconLayer()
    } else {
      const img = new Image(48, 48)
      img.onload = () => {
        if (cancelled) return
        if (!map.hasImage(TREE_ICON_IMAGE)) map.addImage(TREE_ICON_IMAGE, img, { pixelRatio: 2 })
        addIconLayer()
      }
      img.src = TREE_ICON_URL
    }

    return () => {
      cancelled = true
      if (!isMapAlive(map)) return
      for (const id of [iconLayerId, circleLayerId]) {
        if (map.getLayer(id)) map.removeLayer(id)
      }
      if (map.getSource(sourceId)) map.removeSource(sourceId)
    }
  }, [map, sourceId, circleLayerId, iconLayerId, minZoom, circleRadius, iconSize])

  useEffect(() => {
    if (!isMapAlive(map)) return
    map.getSource<GeoJSONSource>(sourceId)?.setData(fc)
  }, [map, sourceId, fc])

  usePointerCursor(circleLayerId, interactive)

  useEffect(() => {
    if (!interactive) return
    const onClick = (e: MapLayerMouseEvent) => {
      const feature = e.features?.[0]
      if (!feature || feature.properties?.disabled) return
      onTreeClick?.(feature.properties?.id as string)
    }
    map.on('click', circleLayerId, onClick)
    return () => {
      map.off('click', circleLayerId, onClick)
    }
  }, [map, circleLayerId, onTreeClick, interactive])
}

export default useTreeMarkerLayer
