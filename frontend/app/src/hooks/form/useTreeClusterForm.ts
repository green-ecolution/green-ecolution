import { QueryClient } from '@tanstack/react-query'
import { clusterQueries } from '@/api/queries'
import type { TreeCluster, TreeClusterCreate, TreeClusterUpdate } from '@/api/backendApi'
import { clusterApi } from '@/api/backendApi'
import { TreeclusterForm } from '@/schema/treeclusterSchema'
import { clusterDraftResolver } from '@green-ecolution/domain-wasm'
import { DefaultValues } from 'react-hook-form'
import { EntityFormConfig, useEntityForm } from './useEntityForm'

const treeClusterConfig: EntityFormConfig<
  TreeclusterForm,
  TreeClusterCreate,
  TreeClusterUpdate,
  TreeCluster
> = {
  formType: 'cluster',
  resolver: clusterDraftResolver<TreeclusterForm>(),

  createFn: (body) => clusterApi.createCluster({ treeClusterCreateRequest: body }),
  updateFn: (id, body) =>
    clusterApi.updateCluster({ clusterId: id, treeClusterUpdateRequest: body }),

  invalidateQueries: (data, queryClient: QueryClient) => {
    queryClient
      .invalidateQueries(clusterQueries.detail(String(data.id)))
      .catch((error) => console.error('Invalidate "clusterQueries.detail" failed:', error))
    queryClient
      .invalidateQueries(clusterQueries.list())
      .catch((error) => console.error('Invalidate "clusterQueries.list" failed:', error))
  },

  successRoute: (id) => ({
    to: '/treecluster/$treeclusterId',
    params: { treeclusterId: id.toString() },
  }),
  replaceOnSuccess: true,
  allowedPaths: [],

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
  opts: {
    clusterId?: string
    initForm?: DefaultValues<TreeclusterForm>
    disableNavigationBlock?: boolean
  },
) => {
  return useEntityForm<TreeclusterForm, TreeClusterCreate, TreeClusterUpdate, TreeCluster>(
    treeClusterConfig,
    mutationType,
    {
      entityId: opts.clusterId,
      initForm: opts.initForm,
      disableNavigationBlock: opts.disableNavigationBlock,
    },
  )
}
