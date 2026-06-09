import { useMutation, useQueryClient } from '@tanstack/react-query'
import { useForm } from 'react-hook-form'
import { clusterDraftResolver } from '@green-ecolution/domain-wasm'
import { clusterApi, type TreeClusterResponse } from '@/api/backendApi'
import { treeClusterIdQuery, treeClusterQuery, clusterMarkersQuery } from '@/api/queries'
import { TreeclusterForm } from '@/schema/treeclusterSchema'
import createToast from '@/hooks/createToast'

export const useClusterPanelEdit = (
  cluster: TreeClusterResponse,
  opts: { onSaved: () => void },
) => {
  const queryClient = useQueryClient()
  const showToast = createToast()

  const form = useForm<TreeclusterForm>({
    resolver: clusterDraftResolver<TreeclusterForm>(),
    defaultValues: {
      name: cluster.name,
      address: cluster.address,
      description: cluster.description,
      soilCondition: cluster.soilCondition,
      treeIds: cluster.trees.map((tree) => tree.id),
    },
  })

  const mutation = useMutation({
    mutationFn: (data: TreeclusterForm) =>
      clusterApi.updateCluster({
        clusterId: cluster.id,
        treeClusterUpdateRequest: {
          name: data.name,
          address: data.address,
          description: data.description,
          soilCondition: data.soilCondition,
          treeIds: data.treeIds,
        },
      }),
    onSuccess: () => {
      void queryClient.invalidateQueries(treeClusterIdQuery(cluster.id))
      void queryClient.invalidateQueries(treeClusterQuery())
      void queryClient.invalidateQueries(clusterMarkersQuery())
      showToast('Die Bewässerungsgruppe wurde erfolgreich bearbeitet.')
      opts.onSaved()
    },
    onError: (error: Error) =>
      showToast(`Fehlermeldung: ${error.message || 'Unbekannter Fehler'}`, 'error'),
  })

  return {
    form,
    onSubmit: form.handleSubmit((data) => mutation.mutate(data)),
    isPending: mutation.isPending,
    isError: mutation.isError,
  }
}
