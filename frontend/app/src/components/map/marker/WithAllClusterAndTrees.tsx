import type { ClusterMarkerResponse, Tree, TreeCluster, TreeMarkerResponse } from '@/api/backendApi'
import useStore from '@/store/store'
import WithAllTrees from './WithAllTrees'
import WithAllClusters from './WithAllClusters'
import WithAllClusterBoundaries from './WithAllClusterBoundaries'
import { useDeferredValue } from 'react'

const defaultSelectedTrees: string[] = []

interface WithTreesAndClustersProps {
  onClickTree?: (tree: TreeMarkerResponse | Tree) => void
  onClickCluster?: (cluster: ClusterMarkerResponse | TreeCluster) => void
  onClickBoundary?: (clusterId: string) => void
  selectedTrees?: string[]
  zoomThreshold?: number
  activeFilter?: boolean
  hasHighlightedTree?: string
  hasHighlightedCluster?: string
}

export const WithTreesAndClusters = ({
  onClickTree,
  onClickCluster,
  onClickBoundary,
  selectedTrees = defaultSelectedTrees,
  zoomThreshold = 17,
  activeFilter = false,
  hasHighlightedTree,
  hasHighlightedCluster,
}: WithTreesAndClustersProps) => {
  const zoom = useStore((state) => state.mapZoom)
  const deferredZoom = useDeferredValue(zoom)
  const showTrees = deferredZoom >= zoomThreshold || activeFilter

  return (
    <>
      {showTrees ? (
        <>
          <WithAllTrees
            onClick={onClickTree}
            selectedTrees={selectedTrees}
            hasHighlightedTree={hasHighlightedTree}
          />
          <WithAllClusterBoundaries onClick={onClickBoundary} />
        </>
      ) : (
        <WithAllClusters
          onClick={onClickCluster}
          highlightedClusters={hasHighlightedCluster ? [hasHighlightedCluster] : []}
        />
      )}
    </>
  )
}
