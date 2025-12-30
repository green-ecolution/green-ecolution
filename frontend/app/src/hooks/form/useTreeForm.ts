import { useMutation, useQueryClient } from '@tanstack/react-query'
import { treeClusterIdQuery, treeIdQuery, treeQuery } from '@/api/queries'
import useToast from '@/hooks/useToast'
import { useNavigate } from '@tanstack/react-router'
import { Tree, TreeCreate, TreeUpdate } from '@green-ecolution/backend-client'
import { treeApi } from '@/api/backendApi'
import { TreeForm, treeSchema } from '@/schema/treeSchema'
import { DefaultValues, useForm } from 'react-hook-form'
import useFormPersist from './usePersistForm'
import { zodResolver } from '@hookform/resolvers/zod'
import { useFormNavigationBlocker } from './useFormNavigationBlocker'

export const useTreeForm = (
  mutationType: 'create' | 'update',
  opts: { treeId?: string; initForm?: DefaultValues<TreeForm> },
) => {
  const showToast = useToast()
  const queryClient = useQueryClient()
  const navigate = useNavigate()

  const form = useForm<TreeForm>({
    defaultValues: opts.initForm,
    resolver: zodResolver(treeSchema),
  })

  const { clear: resetPersist } = useFormPersist(`${mutationType}-tree`, { watch: form.watch })

  const coordsChanged =
    window.sessionStorage.getItem(`${mutationType}-tree-coords-changed`) === 'true'

  const navigationBlocker = useFormNavigationBlocker({
    isDirty: form.formState.isDirty || coordsChanged,
    allowedPaths: ['/map/tree/edit'],
    onLeave: () => {
      window.sessionStorage.removeItem('create-tree')
      window.sessionStorage.removeItem('update-tree')
      window.sessionStorage.removeItem('create-tree-coords-changed')
      window.sessionStorage.removeItem('update-tree-coords-changed')
    },
    message:
      mutationType === 'create'
        ? 'Möchtest du die Seite wirklich verlassen? Deine Eingaben zum Erstellen des Baums gehen verloren, wenn du jetzt gehst.'
        : 'Möchtest du die Seite wirklich verlassen? Deine Änderungen am Baum gehen verloren, wenn du jetzt gehst.',
  })

  const { mutate, isError, error } = useMutation({
    mutationFn: (tree: TreeCreate | TreeUpdate) => {
      if (mutationType === 'create') {
        return treeApi.createTree({
          body: tree as TreeCreate,
        })
      } else if (mutationType === 'update' && opts.treeId) {
        return treeApi.updateTree({
          treeId: Number(opts.treeId),
          body: tree as TreeUpdate,
        })
      }
      return Promise.reject(Error('Invalid mutation type or missing treeId for update'))
    },

    onSuccess: (data: Tree) => {
      resetPersist()
      window.sessionStorage.removeItem('create-tree-coords-changed')
      window.sessionStorage.removeItem('update-tree-coords-changed')
      queryClient
        .invalidateQueries(treeIdQuery(String(data.id)))
        .catch((error) => console.error('Invalidate "treeIdQuery" failed:', error))
      queryClient
        .invalidateQueries(treeQuery())
        .catch((error) => console.error('Invalidate "treeQuery" failed:', error))
      if (data.treeClusterId) {
        queryClient
          .invalidateQueries(treeClusterIdQuery(String(data.treeClusterId)))
          .catch((error) => console.error('Invalidate "treeClusterIdQuery" failed:', error))
      }

      navigationBlocker.allowNavigation()
      navigate({
        to: '/trees/$treeId',
        params: { treeId: data.id.toString() },
      }).catch((error) => console.error('Navigation failed:', error))

      const msg =
        mutationType === 'create'
          ? 'Der Baum wurde erfolgreich erstellt.'
          : 'Der Baum wurde erfolgreich bearbeitet.'
      showToast(msg)
    },

    onError: (error) => {
      console.error('Error with tree mutation:', error)
      showToast(`Fehlermeldung: ${error.message || 'Unbekannter Fehler'}`, 'error')
    },
    throwOnError: true,
  })

  return {
    mutate,
    isError,
    error,
    form,
    navigationBlocker,
  }
}
