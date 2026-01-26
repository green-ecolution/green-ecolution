import { QueryClient } from '@tanstack/react-query'
import { treeClusterIdQuery, treeClusterQuery } from '@/api/queries'
import { TreeCluster, TreeClusterCreate, TreeClusterUpdate } from '@green-ecolution/backend-client'
import { clusterApi } from '@/api/backendApi'
import { clusterSchema, TreeclusterForm } from '@/schema/treeclusterSchema'
import { DefaultValues } from 'react-hook-form'
import { EntityFormConfig, useEntityForm } from './useEntityForm'

const treeClusterConfig: EntityFormConfig<
  TreeclusterForm,
  TreeClusterCreate,
  TreeClusterUpdate,
  TreeCluster
> = {
  formType: 'cluster',
  schema: clusterSchema,

  createFn: (body) => clusterApi.createTreeCluster({ body }),
  updateFn: (id, body) => clusterApi.updateTreeCluster({ clusterId: Number(id), body }),

  invalidateQueries: (data, queryClient: QueryClient) => {
    queryClient
      .invalidateQueries(treeClusterIdQuery(String(data.id)))
      .catch((error) => console.error('Invalidate "treeClusterIdQuery" failed:', error))
    queryClient
      .invalidateQueries(treeClusterQuery())
      .catch((error) => console.error('Invalidate "treeClusterQuery" failed:', error))
  },

  successRoute: (id) => ({
    to: '/treecluster/$treeclusterId',
    params: { treeclusterId: id.toString() },
  }),
  replaceOnSuccess: true,
  allowedPaths: ['/map/treecluster/select/tree'],

  messages: {
    createLeave:
      'Möchtest du die Seite wirklich verlassen? Deine Eingaben zum Erstellen der Bewässerungsgruppe gehen verloren, wenn du jetzt gehst.',
    updateLeave:
      'Möchtest du die Seite wirklich verlassen? Deine Änderungen an der Bewässerungsgruppe gehen verloren, wenn du jetzt gehst.',
    createSuccess: 'Die Bewässerungsgruppe wurde erfolgreich erstellt.',
    updateSuccess: 'Die Bewässerungsgruppe wurde erfolgreich bearbeitet.',
  },
}

export const useTreeClusterForm = (
  mutationType: 'create' | 'update',
  opts: { clusterId?: string; initForm?: DefaultValues<TreeclusterForm> },
) => {
  return useEntityForm<TreeclusterForm, TreeClusterCreate, TreeClusterUpdate, TreeCluster>(
    treeClusterConfig,
    mutationType,
    {
      entityId: opts.clusterId,
      initForm: opts.initForm,
    },
  )
}
