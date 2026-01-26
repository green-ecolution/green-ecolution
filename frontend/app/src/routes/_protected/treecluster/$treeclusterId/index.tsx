import { Loading } from '@green-ecolution/ui'
import TreeClusterDashboard from '@/components/treecluster/TreeClusterDashboard'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'

const treeclusterRoute = getRouteApi('/_protected/treecluster/$treeclusterId')

export const Route = createFileRoute('/_protected/treecluster/$treeclusterId/')({
  component: SingleTreecluster,
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Bewässerungsgruppe wird geladen …" />
  ),
})

function SingleTreecluster() {
  const { treecluster } = treeclusterRoute.useLoaderData()

  return (
    <div className="container mt-6">
      <TreeClusterDashboard treecluster={treecluster} />
    </div>
  )
}

export default SingleTreecluster
