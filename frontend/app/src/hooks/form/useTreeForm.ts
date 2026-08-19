import type { Tree, TreeCreate, TreeUpdate } from '@/api/backendApi'
import { treeApi } from '@/api/backendApi'
import { TreeForm } from '@/schema/treeSchema'
import { treeDraftResolver } from '@green-ecolution/domain-wasm'
import { DefaultValues } from 'react-hook-form'
import { EntityFormConfig, useEntityForm } from './useEntityForm'

const treeConfig: EntityFormConfig<TreeForm, TreeCreate, TreeUpdate, Tree> = {
  formType: 'tree',
  resolver: treeDraftResolver<TreeForm>(),

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
  opts: { treeId?: string; initForm?: DefaultValues<TreeForm>; disableNavigationBlock?: boolean },
) => {
  return useEntityForm<TreeForm, TreeCreate, TreeUpdate, Tree>(treeConfig, mutationType, {
    entityId: opts.treeId,
    initForm: opts.initForm,
    disableNavigationBlock: opts.disableNavigationBlock,
  })
}
