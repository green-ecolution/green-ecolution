import { Loading } from '@green-ecolution/ui'
import TreeClusterUpdate from '@/components/treecluster/TreeClusterUpdate'
import { createFileRoute } from '@tanstack/react-router'
import { useClusterDraft } from '@/store/form/useFormDraft'
import { TreeclusterForm } from '@/schema/treeclusterSchema'

export const Route = createFileRoute('/_protected/treecluster/_formular/$treeclusterId/edit/')({
  component: EditTreeCluster,
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Bewässerungsgruppe wird geladen …" />
  ),
})

function EditTreeCluster() {
  const clusterId = Route.useParams().treeclusterId
  const draft = useClusterDraft<TreeclusterForm>('update')

  const formKey = draft.data?.treeIds?.join(',') ?? 'initial'

  return (
    <div className="container mt-6">
      <TreeClusterUpdate key={formKey} clusterId={clusterId} formState={draft.data} />
    </div>
  )
}
