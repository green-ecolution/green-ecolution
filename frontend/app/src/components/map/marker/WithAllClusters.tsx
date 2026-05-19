import type { ClusterMarkerResponse } from '@/api/backendApi'
import { useSuspenseQuery } from '@tanstack/react-query'
import { clusterMarkersQuery } from '@/api/queries'
import MarkerList from './MarkerList'
import { ClusterIcon } from '../markerIcons'
import { getStatusColor } from '../utils'
import { memo, useCallback, useDeferredValue, useMemo } from 'react'

const defaultHighlighted: string[] = []
const defaultDisabled: string[] = []

const tooltipOptions = {
  direction: 'top' as const,
  offset: [5, -40] as [number, number],
  className: 'font-nunito-sans font-semibold',
}

export interface WithAllClustersProps {
  onClick?: (cluster: ClusterMarkerResponse) => void
  highlightedClusters?: string[]
  disabledClusters?: string[]
}

const WithAllClusters = memo(
  ({
    onClick,
    highlightedClusters = defaultHighlighted,
    disabledClusters = defaultDisabled,
  }: WithAllClustersProps) => {
    const { data } = useSuspenseQuery(clusterMarkersQuery())
    const deferredData = useDeferredValue(data.data)

    const highlightedSet = useMemo(() => new Set(highlightedClusters), [highlightedClusters])
    const disabledSet = useMemo(() => new Set(disabledClusters), [disabledClusters])

    const getIcon = useCallback(
      (c: ClusterMarkerResponse) =>
        ClusterIcon(
          getStatusColor(c.wateringStatus),
          highlightedSet.has(c.id),
          disabledSet.has(c.id),
          c.treeCount,
        ),
      [highlightedSet, disabledSet],
    )

    const getId = useCallback((c: ClusterMarkerResponse) => c.id, [])
    const getTooltip = useCallback((c: ClusterMarkerResponse) => c.name, [])

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

WithAllClusters.displayName = 'WithAllClusters'

export default WithAllClusters
