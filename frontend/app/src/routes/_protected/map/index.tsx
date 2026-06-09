import { createFileRoute, useNavigate } from '@tanstack/react-router'
import type { ClusterMarkerResponse, Tree, TreeCluster, TreeMarkerResponse } from '@/api/backendApi'
import { useCallback } from 'react'
import { z } from 'zod'
import useStore from '@/store/store'
import { WithTreesAndClusters } from '@/components/map/marker/WithAllClusterAndTrees'
import WithAllClusters from '@/components/map/marker/WithAllClusters'
import { filterSearchSchema } from '@/lib/filterSearchSchema'

const mapFilterSchema = filterSearchSchema.pick({ wateringStatuses: true }).extend({
  tree: z.string().optional(),
  cluster: z.string().optional(),
})

function MapView() {
  const navigate = useNavigate({ from: '/map' })
  const search = Route.useSearch()
  const searchTerm = useStore((state) => state.mapSearchTerm)

  const hasActiveFilter = search.wateringStatuses !== undefined

  const handleTreeClick = useCallback(
    (tree: TreeMarkerResponse | Tree) => {
      navigate({ to: `/trees/$treeId`, params: { treeId: tree.id.toString() } }).catch((error) =>
        console.error('Navigation failed:', error),
      )
    },
    [navigate],
  )

  const handleClusterClick = useCallback(
    (cluster: ClusterMarkerResponse | TreeCluster) => {
      navigate({
        to: '/map',
        search: (prev) => ({ ...prev, cluster: cluster.id.toString() }),
      }).catch((error) => console.error('Navigation failed:', error))
    },
    [navigate],
  )

  return hasActiveFilter ? (
    <WithAllClusters
      onClick={handleClusterClick}
      highlightedClusters={search.cluster ? [search.cluster] : []}
      nameFilter={searchTerm}
      statusFilter={search.wateringStatuses}
    />
  ) : (
    <WithTreesAndClusters
      onClickTree={handleTreeClick}
      onClickCluster={handleClusterClick}
      hasHighlightedTree={search.tree}
      hasHighlightedCluster={search.cluster}
      nameFilter={searchTerm}
    />
  )
}

export const Route = createFileRoute('/_protected/map/')({
  component: MapView,
  validateSearch: mapFilterSchema,
})
