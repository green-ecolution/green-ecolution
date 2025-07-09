import { createFileRoute } from '@tanstack/react-router'
import LoadingInfo from '@/components/general/error/LoadingInfo'
import TreeUpdate from '@/components/tree/TreeUpdate'
import { sensorQuery, treeClusterQuery } from '@/api/queries'
import { useSuspenseQuery } from '@tanstack/react-query'

export const Route = createFileRoute('/_protected/trees/_formular/$treeId/edit/')({
  component: EditTree,
  pendingComponent: () => <LoadingInfo label="Baumdaten werden geladen …" />,
  loader: ({ context: { queryClient } }) => {
    queryClient
      .prefetchQuery(sensorQuery())
      .catch((error) => console.error('Prefetching "sensorQuery" failed:', error))
    queryClient
      .prefetchQuery(treeClusterQuery())
      .catch((error) => console.error('Prefetching "treeClusterQuery" failed:', error))
  },
})

function EditTree() {
  const treeId = Route.useParams().treeId
  const { data: sensors } = useSuspenseQuery(sensorQuery())
  const { data: treeClusters } = useSuspenseQuery(treeClusterQuery())

  return (
    <div className="container mt-6">
      <TreeUpdate treeId={treeId} clusters={treeClusters.data} sensors={sensors.data} />
    </div>
  )
}
