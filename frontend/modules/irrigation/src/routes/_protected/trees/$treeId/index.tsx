import { clusterQueries, treeQueries } from '@/api/queries'
import TreeDashboard from '@/components/tree/TreeDashboard'
import { pendingLoading } from '@/lib/router'
import { useQuery, useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'

const treeRoute = getRouteApi('/_protected/trees/$treeId')

export const Route = createFileRoute('/_protected/trees/$treeId/')({
  pendingComponent: pendingLoading({ key: 'tree:detail.loadingLabel' }),
  component: SingleTree,
})

function SingleTree() {
  const { treeId } = treeRoute.useParams()
  // Live query instead of loader data: the watering status changes via
  // MQTT-driven sensor readings and must keep polling.
  const { data: tree } = useSuspenseQuery({
    ...treeQueries.detail(treeId),
    refetchInterval: 30_000,
    refetchOnWindowFocus: true,
  })
  const { data: treeCluster } = useQuery({
    ...clusterQueries.detail(tree.treeClusterId ?? ''),
    enabled: tree.treeClusterId != null,
  })

  return (
    <div className="container mt-6">
      <TreeDashboard tree={tree} treeCluster={treeCluster} />
    </div>
  )
}
