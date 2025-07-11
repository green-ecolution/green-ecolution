import { Tree, TreeClusterInList, TreeCluster } from '@green-ecolution/backend-client'
import useStore from '@/store/store'
import WithAllTrees from './WithAllTrees'
import WithAllClusters from './WithAllClusters'

const defaultSelectedTrees: number[] = []

interface WithTreesAndClustersProps {
  onClickTree?: (tree: Tree) => void
  onClickCluster?: (cluster: TreeClusterInList | TreeCluster) => void
  selectedTrees?: number[]
  zoomThreshold?: number
  activeFilter?: boolean
  hasHighlightedTree?: number
  hasHighlightedCluster?: number
}

export const WithTreesAndClusters = ({
  onClickTree,
  onClickCluster,
  selectedTrees = defaultSelectedTrees,
  zoomThreshold = 17,
  activeFilter = false,
  hasHighlightedTree,
  hasHighlightedCluster,
}: WithTreesAndClustersProps) => {
  const { zoom } = useStore((state) => ({
    zoom: state.map.zoom,
  }))

  return (
    <>
      {zoom >= zoomThreshold || activeFilter ? (
        <WithAllTrees
          onClick={onClickTree}
          selectedTrees={selectedTrees}
          hasHighlightedTree={hasHighlightedTree}
        />
      ) : (
        <WithAllClusters
          onClick={onClickCluster}
          highlightedClusters={hasHighlightedCluster ? [hasHighlightedCluster] : []}
        />
      )}
    </>
  )
}
