import TreeClusterDashboard from '@/components/treecluster/TreeClusterDashboard'
import { treeClusterIdQuery } from '@/api/queries'
import { pendingLoading } from '@/lib/router'
import { useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'

const treeclusterRoute = getRouteApi('/_protected/treecluster/$treeclusterId')

export const Route = createFileRoute('/_protected/treecluster/$treeclusterId/')({
  component: SingleTreecluster,
  pendingComponent: pendingLoading('Bewässerungsgruppe wird geladen …'),
})

function SingleTreecluster() {
  const { treeclusterId } = treeclusterRoute.useParams()
  // Live query instead of loader data: cluster status changes via MQTT-driven
  // sensor readings and must keep polling.
  const { data: treecluster } = useSuspenseQuery({
    ...treeClusterIdQuery(treeclusterId),
    refetchInterval: 30_000,
    refetchOnWindowFocus: true,
  })

  return (
    <div className="container mt-6">
      <TreeClusterDashboard treecluster={treecluster} />
    </div>
  )
}
