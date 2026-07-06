import { treeClusterIdQuery } from '@/api/queries'
import TreeDashboard from '@/components/tree/TreeDashboard'
import { pendingLoading } from '@/lib/router'
import { useQuery } from '@tanstack/react-query'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'

const treeRoute = getRouteApi('/_protected/trees/$treeId')

export const Route = createFileRoute('/_protected/trees/$treeId/')({
  pendingComponent: pendingLoading('Baumdaten werden geladen …'),
  component: SingleTree,
})

function SingleTree() {
  const { tree } = treeRoute.useLoaderData()
  const { data: treeCluster } = useQuery({
    ...treeClusterIdQuery(tree.treeClusterId?.toString() ?? ''),
    enabled: tree.treeClusterId !== undefined,
  })

  return (
    <div className="container mt-6">
      <TreeDashboard tree={tree} treeCluster={treeCluster} />
    </div>
  )
}
