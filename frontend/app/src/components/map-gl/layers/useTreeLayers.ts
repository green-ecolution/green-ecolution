import { useEffect } from 'react'
import { useQuery } from '@tanstack/react-query'
import type { GeoJSONSource, MapLayerMouseEvent } from 'maplibre-gl'
import type { FeatureCollection, Point } from 'geojson'
import type { TreeMarkerResponse, WateringStatus } from '@green-ecolution/backend-client'
import { treeMarkersQuery } from '@/api/queries'
import useStore from '@/store/store'
import { useMaplibreMap } from '../MapContext'
import {
  LAYERS,
  SOURCES,
  STATUS_COLOR_EXPRESSION,
  TREE_ICON_IMAGE,
  TREE_ICON_URL,
  TREE_ZOOM_THRESHOLD,
} from '../mapStyle'
import useViewportBBox from '../hooks/useViewportBBox'
import { isMapAlive } from '../mapReady'

const treesToFC = (trees: TreeMarkerResponse[]): FeatureCollection<Point> => ({
  type: 'FeatureCollection',
  features: trees.map((t) => ({
    type: 'Feature',
    geometry: { type: 'Point', coordinates: [t.longitude, t.latitude] },
    properties: { id: t.id, status: t.wateringStatus },
  })),
})

export interface UseTreeLayersOptions {
  onTreeClick?: (treeId: string) => void
  wateringStatuses?: WateringStatus[]
  interactive?: boolean
}

const useTreeLayers = ({
  onTreeClick,
  wateringStatuses,
  interactive = true,
}: UseTreeLayersOptions = {}) => {
  const map = useMaplibreMap()
  const bbox = useViewportBBox()
  const zoom = useStore((s) => s.mapZoom)
  // Only the detailed zoom levels show individual trees, so skip the fetch below it.
  const { data } = useQuery({
    ...treeMarkersQuery({ bbox, wateringStatuses }),
    enabled: zoom >= TREE_ZOOM_THRESHOLD,
  })

  useEffect(() => {
    let cancelled = false

    if (!map.getSource(SOURCES.treePoints)) {
      map.addSource(SOURCES.treePoints, {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: [] },
      })
    }
    if (!map.getLayer(LAYERS.treePoints)) {
      map.addLayer({
        id: LAYERS.treePoints,
        type: 'circle',
        source: SOURCES.treePoints,
        minzoom: TREE_ZOOM_THRESHOLD,
        paint: {
          'circle-radius': ['interpolate', ['linear'], ['zoom'], 17, 9, 19, 14, 22, 18],
          'circle-color': STATUS_COLOR_EXPRESSION,
          'circle-stroke-width': 2,
          'circle-stroke-color': '#ffffff',
        },
      })
    }

    const addIconLayer = () => {
      if (cancelled || map.getLayer(LAYERS.treeIcon)) return
      map.addLayer({
        id: LAYERS.treeIcon,
        type: 'symbol',
        source: SOURCES.treePoints,
        minzoom: TREE_ZOOM_THRESHOLD,
        layout: {
          'icon-image': TREE_ICON_IMAGE,
          'icon-size': ['interpolate', ['linear'], ['zoom'], 17, 0.46, 19, 0.72, 22, 0.95],
          'icon-allow-overlap': true,
          'icon-ignore-placement': true,
        },
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
      for (const id of [LAYERS.treeIcon, LAYERS.treePoints]) {
        if (map.getLayer(id)) map.removeLayer(id)
      }
      if (map.getSource(SOURCES.treePoints)) map.removeSource(SOURCES.treePoints)
    }
  }, [map])

  useEffect(() => {
    if (!isMapAlive(map)) return
    map.getSource<GeoJSONSource>(SOURCES.treePoints)?.setData(treesToFC(data?.data ?? []))
  }, [map, data])

  useEffect(() => {
    if (!interactive) return
    const onClick = (e: MapLayerMouseEvent) => {
      const feature = e.features?.[0]
      if (!feature) return
      onTreeClick?.(feature.properties?.id as string)
    }
    const enter = () => {
      map.getCanvas().style.cursor = 'pointer'
    }
    const leave = () => {
      map.getCanvas().style.cursor = ''
    }
    map.on('click', LAYERS.treePoints, onClick)
    map.on('mouseenter', LAYERS.treePoints, enter)
    map.on('mouseleave', LAYERS.treePoints, leave)
    return () => {
      map.off('click', LAYERS.treePoints, onClick)
      map.off('mouseenter', LAYERS.treePoints, enter)
      map.off('mouseleave', LAYERS.treePoints, leave)
    }
  }, [map, onTreeClick, interactive])
}

export default useTreeLayers
