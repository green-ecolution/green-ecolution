import type { TreeMarkerResponse } from '@/api/backendApi'
import { useQuery } from '@tanstack/react-query'
import { treeMarkersQuery } from '@/api/queries'
import { useViewportBBox } from '@/hooks/useViewportBBox'
import { TreeMarkerIcon } from '../markerIcons'
import MarkerList from './MarkerList'
import { getStatusColor } from '../utils'
import { useDeferredValue } from 'react'

const defaultSelectedTrees: string[] = []
const emptyMarkers: TreeMarkerResponse[] = []

const tooltipOptions = {
  direction: 'top' as const,
  offset: [5, -40] as [number, number],
  className: 'font-nunito-sans font-semibold',
}

const getId = (t: TreeMarkerResponse) => t.id
const getTooltip = (t: TreeMarkerResponse) => t.number

export interface WithAllTreesProps {
  onClick?: (tree: TreeMarkerResponse) => void
  selectedTrees?: string[]
  hasHighlightedTree?: string
}

const WithAllTrees = ({
  onClick,
  selectedTrees = defaultSelectedTrees,
  hasHighlightedTree,
}: WithAllTreesProps) => {
  const bbox = useViewportBBox()
  const { data } = useQuery({
    ...treeMarkersQuery({ bbox: bbox ?? { swLat: 0, swLng: 0, neLat: 0.0001, neLng: 0.0001 } }),
    // enabled: bbox !== null prevents firing requests before the map ref is ready
    enabled: bbox !== null,
  })

  const deferredData = useDeferredValue(data?.data ?? emptyMarkers)

  const selectedSet = new Set(selectedTrees)

  const getIcon = (t: TreeMarkerResponse) =>
    TreeMarkerIcon(
      getStatusColor(t.wateringStatus),
      selectedSet.has(t.id),
      hasHighlightedTree === t.id,
      false,
    )

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
}

WithAllTrees.displayName = 'WithAllTrees'

export default WithAllTrees
