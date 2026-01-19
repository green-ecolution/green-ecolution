import { Loading } from '@green-ecolution/ui'
import TreeClusterUpdate from '@/components/treecluster/TreeClusterUpdate'
import useStore from '@/store/store'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/treecluster/_formular/$treeclusterId/edit/')({
  component: EditTreeCluster,
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Bewässerungsgruppe wird geladen …" />
  ),
  loader: () => {
    if (!useStore.getState().isAuthenticated) return
  },
})

function EditTreeCluster() {
  const clusterId = Route.useParams().treeclusterId

  return (
    <div className="container mt-6">
      <TreeClusterUpdate clusterId={clusterId} />
    </div>
  )
}
