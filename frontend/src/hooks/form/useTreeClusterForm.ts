import { useMutation, useQueryClient } from '@tanstack/react-query'
import { treeClusterIdQuery, treeClusterQuery } from '@/api/queries'
import useToast from '@/hooks/useToast'
import { useBlocker, useNavigate } from '@tanstack/react-router'
import { TreeCluster, TreeClusterCreate, TreeClusterUpdate } from '@green-ecolution/backend-client'
import { clusterApi } from '@/api/backendApi'
import { clusterSchema, TreeclusterForm } from '@/schema/treeclusterSchema'
import { DefaultValues, useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import useFormPersist from './usePersistForm'

export const useTreeClusterForm = (
  mutationType: 'create' | 'update',
  opts: { clusterId?: string; initForm?: DefaultValues<TreeclusterForm> },
) => {
  const showToast = useToast()
  const queryClient = useQueryClient()
  const navigate = useNavigate()

  const form = useForm<TreeclusterForm>({
    defaultValues: opts.initForm,
    resolver: zodResolver(clusterSchema),
  })

  const { clear: resetPersist } = useFormPersist(`${mutationType}-cluster`, { watch: form.watch })

  // TODO: only on isDirty
  useBlocker({
    shouldBlockFn: ({ next }) => {
      if (next.pathname === '/map/treecluster/select/tree') return false

      const shouldLeave = confirm(
        mutationType === 'create'
          ? 'Möchtest du die Seite wirklich verlassen? Deine Eingaben zum Erstellen der Bewässerungsgruppe gehen verloren, wenn du jetzt gehst.'
          : 'Möchtest du die Seite wirklich verlassen? Deine Änderungen an der Bewässerungsgruppe gehen verloren, wenn du jetzt gehst.',
      )

      if (shouldLeave) {
        window.sessionStorage.removeItem('create-cluster')
        window.sessionStorage.removeItem('update-cluster')
      }

      return !shouldLeave
    },
  })

  const { mutate, isError, error } = useMutation({
    mutationFn: (cluster: TreeClusterCreate | TreeClusterUpdate) => {
      if (mutationType === 'create') {
        return clusterApi.createTreeCluster({
          body: cluster as TreeClusterCreate,
        })
      } else if (mutationType === 'update' && opts.clusterId) {
        return clusterApi.updateTreeCluster({
          clusterId: Number(opts.clusterId),
          body: cluster as TreeClusterUpdate,
        })
      }
      return Promise.reject(Error('Invalid mutation type or missing clusterId for update'))
    },

    onSuccess: (data: TreeCluster) => {
      resetPersist()
      queryClient
        .invalidateQueries(treeClusterIdQuery(String(data.id)))
        .catch((error) => console.error('Invalidate "treeClusterIdQuery" failed:', error))
      queryClient
        .invalidateQueries(treeClusterQuery())
        .catch((error) => console.error('Invalidate "treeClusterQuery" failed:', error))
      navigate({
        to: '/treecluster/$treeclusterId',
        params: { treeclusterId: data.id.toString() },
        search: { resetStore: false },
        replace: true,
      }).catch((error) => console.error('Navigation failed:', error))

      if (mutationType === 'create') showToast('Die Bewässerungsgruppe wurde erfolgreich erstellt.')
      else showToast('Die Bewässerungsgruppe wurde erfolgreich bearbeitet.')
    },

    onError: (error) => {
      console.error('Error with tree cluster mutation:', error)
      showToast(`Fehlermeldung: ${error.message || 'Unbekannter Fehler'}`, 'error')
    },
    throwOnError: true,
  })

  return {
    mutate: mutate,
    isError: isError,
    error: error,
    form,
  }
}
