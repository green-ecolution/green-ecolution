import type { TreeMarkerResponse } from '@/api/backendApi'
import { useQuery } from '@tanstack/react-query'
import { treeMarkersQuery } from '@/api/queries'
import { useViewportBBox } from '@/hooks/useViewportBBox'
import { TreeMarkerIcon } from '../markerIcons'
import MarkerList from './MarkerList'
import { getStatusColor } from '../utils'
import { memo, useCallback, useDeferredValue, useMemo } from 'react'

const defaultSelectedTrees: string[] = []
const emptyMarkers: TreeMarkerResponse[] = []

const tooltipOptions = {
  direction: 'top' as const,
  offset: [5, -40] as [number, number],
  className: 'font-nunito-sans font-semibold',
}

export interface WithAllTreesProps {
  onClick?: (tree: TreeMarkerResponse) => void
  selectedTrees?: string[]
  hasHighlightedTree?: string
}

const WithAllTrees = memo(
  ({ onClick, selectedTrees = defaultSelectedTrees, hasHighlightedTree }: WithAllTreesProps) => {
    const bbox = useViewportBBox()
    const { data } = useQuery({
      ...treeMarkersQuery({ bbox: bbox ?? { swLat: 0, swLng: 0, neLat: 0.0001, neLng: 0.0001 } }),
      // enabled: bbox !== null prevents firing requests before the map ref is ready
      enabled: bbox !== null,
    })

    const deferredData = useDeferredValue(data?.data ?? emptyMarkers)

    const selectedSet = useMemo(() => new Set(selectedTrees), [selectedTrees])

    const getIcon = useCallback(
      (t: TreeMarkerResponse) =>
        TreeMarkerIcon(
          getStatusColor(t.wateringStatus),
          selectedSet.has(t.id),
          hasHighlightedTree === t.id,
          false,
        ),
      [selectedSet, hasHighlightedTree],
    )

    const getId = useCallback((t: TreeMarkerResponse) => t.id, [])
    const getTooltip = useCallback((t: TreeMarkerResponse) => t.number, [])

    return (
      <MarkerList
        data={deferredData}
        onClick={onClick}
        icon={getIcon}
        getId={getId}
        tooltipContent={getTooltip}
        tooltipOptions={tooltipOptions}
      />
    )
  },
)

WithAllTrees.displayName = 'WithAllTrees'

export default WithAllTrees
