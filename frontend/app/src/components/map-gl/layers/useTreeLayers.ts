import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import type { TreeMarkerResponse, WateringStatus } from '@green-ecolution/backend-client'
import { treeMarkersQuery } from '@/api/queries'
import useStore from '@/store/store'
import { LAYERS, SOURCES, TREE_ZOOM_THRESHOLD } from '../mapStyle'
import useViewportBBox from '../hooks/useViewportBBox'
import useTreeMarkerLayer, { type TreeMarkerPoint } from './useTreeMarkerLayer'

export interface UseTreeLayersOptions {
  onTreeClick?: (treeId: string) => void
  wateringStatuses?: WateringStatus[]
  interactive?: boolean
}

const toPoints = (trees: TreeMarkerResponse[]): TreeMarkerPoint[] =>
  trees.map((t) => ({
    id: t.id,
    longitude: t.longitude,
    latitude: t.latitude,
    status: t.wateringStatus,
  }))

const useTreeLayers = ({
  onTreeClick,
  wateringStatuses,
  interactive = true,
}: UseTreeLayersOptions = {}) => {
  const bbox = useViewportBBox()
  const zoom = useStore((s) => s.mapZoom)
  // Only the detailed zoom levels show individual trees, so skip the fetch below it.
  const { data } = useQuery({
    ...treeMarkersQuery({ bbox, wateringStatuses }),
    enabled: zoom >= TREE_ZOOM_THRESHOLD,
  })
  const trees = useMemo(() => toPoints(data?.data ?? []), [data])

  useTreeMarkerLayer({
    trees,
    sourceId: SOURCES.treePoints,
    circleLayerId: LAYERS.treePoints,
    iconLayerId: LAYERS.treeIcon,
    minZoom: TREE_ZOOM_THRESHOLD,
    onTreeClick,
    interactive,
  })
}

export default useTreeLayers
