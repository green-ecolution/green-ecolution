import { treeClusterIdQuery, treeIdQuery } from '@/api/queries'
import { Loading } from '@green-ecolution/ui'
import TreeDashboard from '@/components/tree/TreeDashboard'
import { useQuery, useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/trees/$treeId/')({
  pendingComponent: () => <Loading className="mt-20 justify-center" label="Baumdaten werden geladen …" />,
  component: SingleTree,
  loader: ({ context: { queryClient }, params }) =>
    queryClient.prefetchQuery(treeIdQuery(params.treeId)),
})

function SingleTree() {
  const treeId = Route.useParams().treeId
  const { data: tree } = useSuspenseQuery(treeIdQuery(treeId))
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
