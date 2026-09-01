import type { Tree, TreeCreate, TreeUpdate } from '@/api/backendApi'
import { treeApi } from '@/api/backendApi'
import { useIssueTranslator } from '@/lib/i18n/validation'
import { TreeForm } from '@/schema/treeSchema'
import { treeDraftResolver } from '@green-ecolution/domain-wasm'
import { DefaultValues } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { EntityFormConfig, useEntityForm } from './useEntityForm'

export const useTreeForm = (
  mutationType: 'create' | 'update',
  opts: { treeId?: string; initForm?: DefaultValues<TreeForm>; disableNavigationBlock?: boolean },
) => {
  const translate = useIssueTranslator()
  const { t } = useTranslation('tree')

  const treeConfig: EntityFormConfig<TreeForm, TreeCreate, TreeUpdate, Tree> = {
    formType: 'tree',
    resolver: treeDraftResolver<TreeForm>(translate),

    createFn: (body) => treeApi.createTree({ treeCreateRequest: body }),
    updateFn: (id, body) => treeApi.updateTree({ treeId: id, treeUpdateRequest: body }),

    // Cluster too: assigning a tree shifts the cluster's centroid and status.
    invalidates: ['tree', 'cluster'],

    successRoute: (id) => ({
      to: '/trees/$treeId',
      params: { treeId: id.toString() },
    }),
    allowedPaths: [],

    messages: {
      createLeave: t('form.createLeaveMessage'),
      updateLeave: t('form.updateLeaveMessage'),
      createSuccess: t('form.createSuccessMessage'),
      updateSuccess: t('form.updateSuccessMessage'),
    },
  }

  return useEntityForm<TreeForm, TreeCreate, TreeUpdate, Tree>(treeConfig, mutationType, {
    entityId: opts.treeId,
    initForm: opts.initForm,
    disableNavigationBlock: opts.disableNavigationBlock,
  })
}
