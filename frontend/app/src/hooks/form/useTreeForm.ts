import { QueryClient } from '@tanstack/react-query'
import { treeClusterIdQuery, treeIdQuery, treeQuery } from '@/api/queries'
import { Tree, TreeCreate, TreeUpdate } from '@green-ecolution/backend-client'
import { treeApi } from '@/api/backendApi'
import { TreeForm, treeSchema } from '@/schema/treeSchema'
import { DefaultValues } from 'react-hook-form'
import { EntityFormConfig, useEntityForm } from './useEntityForm'

const treeConfig: EntityFormConfig<TreeForm, TreeCreate, TreeUpdate, Tree> = {
  formType: 'tree',
  schema: treeSchema,

  createFn: (body) => treeApi.createTree({ body }),
  updateFn: (id, body) => treeApi.updateTree({ treeId: Number(id), body }),

  invalidateQueries: (data, queryClient: QueryClient) => {
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
  },

  successRoute: (id) => ({
    to: '/trees/$treeId',
    params: { treeId: id.toString() },
  }),
  allowedPaths: ['/map/tree/edit'],

  messages: {
    createLeave:
      'Möchtest du die Seite wirklich verlassen? Deine Eingaben zum Erstellen des Baums gehen verloren, wenn du jetzt gehst.',
    updateLeave:
      'Möchtest du die Seite wirklich verlassen? Deine Änderungen am Baum gehen verloren, wenn du jetzt gehst.',
    createSuccess: 'Der Baum wurde erfolgreich erstellt.',
    updateSuccess: 'Der Baum wurde erfolgreich bearbeitet.',
  },
}

export const useTreeForm = (
  mutationType: 'create' | 'update',
  opts: { treeId?: string; initForm?: DefaultValues<TreeForm> },
) => {
  return useEntityForm<TreeForm, TreeCreate, TreeUpdate, Tree>(treeConfig, mutationType, {
    entityId: opts.treeId,
    initForm: opts.initForm,
  })
}
