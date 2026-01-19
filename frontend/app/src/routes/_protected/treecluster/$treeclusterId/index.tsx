import { treeClusterIdQuery } from '@/api/queries'
import { Loading } from '@green-ecolution/ui'
import TreeClusterDashboard from '@/components/treecluster/TreeClusterDashboard'
import { useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/treecluster/$treeclusterId/')({
  component: SingleTreecluster,
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Bewässerungsgruppe wird geladen …" />
  ),
  loader: ({ context: { queryClient }, params }) =>
    queryClient.prefetchQuery(treeClusterIdQuery(params.treeclusterId)),
})

function SingleTreecluster() {
  const clusterId = Route.useParams().treeclusterId
  const { data: treecluster } = useSuspenseQuery(treeClusterIdQuery(clusterId))

  return (
    <div className="container mt-6">
      <TreeClusterDashboard treecluster={treecluster} />
    </div>
  )
}

export default SingleTreecluster
