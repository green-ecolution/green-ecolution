import type { ClusterMarkerResponse } from '@/api/backendApi'
import { useSuspenseQuery } from '@tanstack/react-query'
import { clusterMarkersQuery } from '@/api/queries'
import MarkerList from './MarkerList'
import { ClusterIcon } from '../markerIcons'
import { getStatusColor } from '../utils'
import { useDeferredValue } from 'react'

const defaultHighlighted: string[] = []
const defaultDisabled: string[] = []

const tooltipOptions = {
  direction: 'top' as const,
  offset: [5, -40] as [number, number],
  className: 'font-nunito-sans font-semibold',
}

const getId = (c: ClusterMarkerResponse) => c.id
const getTooltip = (c: ClusterMarkerResponse) => c.name

export interface WithAllClustersProps {
  onClick?: (cluster: ClusterMarkerResponse) => void
  highlightedClusters?: string[]
  disabledClusters?: string[]
}

const WithAllClusters = ({
  onClick,
  highlightedClusters = defaultHighlighted,
  disabledClusters = defaultDisabled,
}: WithAllClustersProps) => {
  const { data } = useSuspenseQuery(clusterMarkersQuery())
  const deferredData = useDeferredValue(data.data)

  const highlightedSet = new Set(highlightedClusters)
  const disabledSet = new Set(disabledClusters)

  const getIcon = (c: ClusterMarkerResponse) =>
    ClusterIcon(
      getStatusColor(c.wateringStatus),
      highlightedSet.has(c.id),
      disabledSet.has(c.id),
      c.treeCount,
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

WithAllClusters.displayName = 'WithAllClusters'

export default WithAllClusters
