import { TreeClusterInList } from '@green-ecolution/backend-client'
import { useSuspenseQuery } from '@tanstack/react-query'
import { treeClusterQuery } from '@/api/queries'
import MarkerList from './MarkerList'
import { ClusterIcon } from '../MapMarker'
import { getStatusColor } from '../utils'

export interface WithAllClustersProps {
  onClick?: (tree: TreeClusterInList) => void
  highlightedClusters?: number[]
  disabledClusters?: number[]
}

const WithAllClusters = ({
  onClick,
  highlightedClusters,
  disabledClusters,
}: WithAllClustersProps) => {
  const { data } = useSuspenseQuery(treeClusterQuery())

  return (
    <MarkerList
      data={data.data.filter(
        (cluster) =>
          cluster.latitude !== null && cluster.longitude !== null && cluster.treeIds !== undefined,
      )}
      onClick={onClick}
      icon={(c: TreeClusterInList) =>
        ClusterIcon(
          getStatusColor(c.wateringStatus),
          highlightedClusters?.includes(c.id) ?? false,
          disabledClusters?.includes(c.id) ?? false,
          c.treeIds?.length ?? 0,
        )
      }
      tooltipContent={(c) => c.name}
      tooltipOptions={{
        direction: 'top',
        offset: [5, -40],
        className: 'font-nunito-sans font-semibold',
      }}
    />
  )
}

export default WithAllClusters
