import { Loading } from '@green-ecolution/ui'
import TreeClusterUpdate from '@/components/treecluster/TreeClusterUpdate'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'
import { useClusterDraft } from '@/store/form/useFormDraft'
import { TreeclusterForm } from '@/schema/treeclusterSchema'

const treeclusterFormRoute = getRouteApi('/_protected/treecluster/_formular/$treeclusterId')

export const Route = createFileRoute('/_protected/treecluster/_formular/$treeclusterId/edit/')({
  component: EditTreeCluster,
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Bewässerungsgruppe wird geladen …" />
  ),
})

function EditTreeCluster() {
  const { treecluster } = treeclusterFormRoute.useLoaderData()
  const draft = useClusterDraft<TreeclusterForm>('update')

  const formKey = draft.data?.treeIds?.join(',') ?? 'initial'

  return (
    <div className="container mt-6">
      <TreeClusterUpdate
        key={formKey}
        clusterId={treecluster.id.toString()}
        formState={draft.data}
      />
    </div>
  )
}
