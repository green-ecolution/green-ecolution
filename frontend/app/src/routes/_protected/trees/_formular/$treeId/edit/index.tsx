import { createFileRoute, getRouteApi } from '@tanstack/react-router'
import { Loading } from '@green-ecolution/ui'
import TreeUpdate from '@/components/tree/TreeUpdate'
import { sensorQuery, treeClusterQuery } from '@/api/queries'
import { useSuspenseQuery } from '@tanstack/react-query'
import { useTreeDraft } from '@/store/form/useFormDraft'
import { TreeForm } from '@/schema/treeSchema'

const treeFormRoute = getRouteApi('/_protected/trees/_formular/$treeId')

export const Route = createFileRoute('/_protected/trees/_formular/$treeId/edit/')({
  component: EditTree,
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Baumdaten werden geladen …" />
  ),
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
  const { tree } = treeFormRoute.useLoaderData()
  const draft = useTreeDraft<TreeForm>('update')
  const { data: sensors } = useSuspenseQuery(sensorQuery())
  const { data: treeClusters } = useSuspenseQuery(treeClusterQuery())

  const formKey = `${draft.data?.latitude ?? 'initial'}-${draft.data?.longitude ?? 'initial'}`

  return (
    <div className="container mt-6">
      <TreeUpdate
        key={formKey}
        treeId={tree.id.toString()}
        clusters={treeClusters.data}
        sensors={sensors.data}
      />
    </div>
  )
}
