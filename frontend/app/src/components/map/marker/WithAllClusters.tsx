import { TreeClusterInList } from '@green-ecolution/backend-client'
import { useSuspenseQuery } from '@tanstack/react-query'
import { treeClusterQuery } from '@/api/queries'
import MarkerList from './MarkerList'
import { ClusterIcon } from '../MapMarker'
import { getStatusColor } from '../utils'
import { memo, useCallback, useMemo } from 'react'

const defaultHighlighted: number[] = []
const defaultDisabled: number[] = []

const tooltipOptions = {
  direction: 'top' as const,
  offset: [5, -40] as [number, number],
  className: 'font-nunito-sans font-semibold',
}

export interface WithAllClustersProps {
  onClick?: (cluster: TreeClusterInList) => void
  highlightedClusters?: number[]
  disabledClusters?: number[]
}

const WithAllClusters = memo(
  ({
    onClick,
    highlightedClusters = defaultHighlighted,
    disabledClusters = defaultDisabled,
  }: WithAllClustersProps) => {
    const { data } = useSuspenseQuery(treeClusterQuery())

    const filteredData = useMemo(
      () =>
        data.data.filter(
          (cluster) =>
            cluster.latitude !== null &&
            cluster.longitude !== null &&
            cluster.treeIds !== undefined,
        ),
      [data.data],
    )

    const highlightedSet = useMemo(() => new Set(highlightedClusters), [highlightedClusters])
    const disabledSet = useMemo(() => new Set(disabledClusters), [disabledClusters])

    const getIcon = useCallback(
      (c: TreeClusterInList) =>
        ClusterIcon(
          getStatusColor(c.wateringStatus),
          highlightedSet.has(c.id),
          disabledSet.has(c.id),
          c.treeIds?.length ?? 0,
        ),
      [highlightedSet, disabledSet],
    )

    const getId = useCallback((c: TreeClusterInList) => c.id, [])
    const getTooltip = useCallback((c: TreeClusterInList) => c.name, [])

    return (
      <MarkerList
        data={filteredData}
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
