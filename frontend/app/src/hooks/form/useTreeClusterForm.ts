import type { TreeCluster, TreeClusterCreate, TreeClusterUpdate } from '@/api/backendApi'
import { clusterApi } from '@/api/backendApi'
import { useIssueTranslator } from '@/lib/i18n/validation'
import { TreeclusterForm } from '@/schema/treeclusterSchema'
import { clusterDraftResolver } from '@green-ecolution/domain-wasm'
import { DefaultValues } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { EntityFormConfig, useEntityForm } from './useEntityForm'

export const useTreeClusterForm = (
  mutationType: 'create' | 'update',
  opts: {
    clusterId?: string
    initForm?: DefaultValues<TreeclusterForm>
    disableNavigationBlock?: boolean
  },
) => {
  const translate = useIssueTranslator()
  const { t } = useTranslation('treecluster')

  const treeClusterConfig: EntityFormConfig<
    TreeclusterForm,
    TreeClusterCreate,
    TreeClusterUpdate,
    TreeCluster
  > = {
    formType: 'cluster',
    resolver: clusterDraftResolver<TreeclusterForm>(translate),

    createFn: (body) => clusterApi.createCluster({ treeClusterCreateRequest: body }),
    updateFn: (id, body) =>
      clusterApi.updateCluster({ clusterId: id, treeClusterUpdateRequest: body }),

    // Tree too: replacing the cluster's trees changes their cluster membership.
    invalidates: ['cluster', 'tree'],

    successRoute: (id) => ({
      to: '/treecluster/$treeclusterId',
      params: { treeclusterId: id.toString() },
    }),
    replaceOnSuccess: true,
    allowedPaths: [],

    messages: {
      createLeave: t('form.createLeaveMessage'),
      updateLeave: t('form.updateLeaveMessage'),
      createSuccess: t('form.createSuccessMessage'),
      updateSuccess: t('form.updateSuccessMessage'),
    },
  }

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
