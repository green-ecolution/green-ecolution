import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useCallback, useState } from 'react'
import { FormProvider, useWatch, type DefaultValues, type SubmitHandler } from 'react-hook-form'
import { useTranslation } from 'react-i18next'
import { SoilCondition } from '@/api/backendApi'
import { TreeclusterForm } from '@/schema/treeclusterSchema'
import FormForTreecluster from '@/components/general/form/FormForTreecluster'
import ForeignTreeDialog from '@/components/general/form/ForeignTreeDialog'
import UnsavedChangesDialog from '@/components/general/form/UnsavedChangesDialog'
import { useTreeClusterForm } from '@/hooks/form/useTreeClusterForm'
import { useClusterOrganizationSelection } from '@/hooks/form/useClusterOrganizationSelection'
import { forbiddenErrorComponent, requirePermission } from '@/lib/router'
import MapPanel from '@/components/map-gl/MapPanel'
import useClusterBoundaryLayer from '@/components/map-gl/layers/useClusterBoundaryLayer'
import useSelectableTreeLayer, {
  type ForeignTree,
} from '@/components/map-gl/layers/useSelectableTreeLayer'

export const Route = createFileRoute('/_protected/map/treecluster/new/')({
  component: NewClusterOnMap,
  beforeLoad: requirePermission(['tree_cluster:create']),
  errorComponent: forbiddenErrorComponent(),
})

const defaultForm: DefaultValues<TreeclusterForm> = {
  soilCondition: SoilCondition.Unknown,
  treeIds: [],
}

function NewClusterOnMap() {
  const { t } = useTranslation('map')
  const navigate = useNavigate({ from: Route.fullPath })
  const { mutate, isError, error, form, navigationBlocker, saveDraft } = useTreeClusterForm(
    'create',
    {
      initForm: defaultForm,
    },
  )
  const treeIds = useWatch({ control: form.control, name: 'treeIds' }) ?? []
  const {
    organizations,
    organizationId,
    changeOrganization,
    discardedTreeCount,
    nameOf,
    canCreateIn,
  } = useClusterOrganizationSelection(form)
  const [foreignTree, setForeignTree] = useState<ForeignTree | null>(null)

  const toggleTree = useCallback(
    (id: string) => {
      const current = form.getValues('treeIds') ?? []
      const next = current.includes(id) ? current.filter((x) => x !== id) : [...current, id]
      form.setValue('treeIds', next, { shouldValidate: true, shouldDirty: true })
    },
    [form],
  )

  // Switching drops the previous selection, so the clicked tree is re-added
  // afterwards — it is the reason the user switched in the first place.
  const adoptForeignTree = () => {
    if (!foreignTree) return
    changeOrganization(foreignTree.organizationId)
    form.setValue('treeIds', [foreignTree.id], { shouldValidate: true, shouldDirty: true })
    setForeignTree(null)
  }

  useClusterBoundaryLayer({ interactive: false })
  useSelectableTreeLayer({
    selectedIds: treeIds,
    onToggle: toggleTree,
    organizationId,
    onForeignTree: setForeignTree,
  })

  const onSubmit: SubmitHandler<TreeclusterForm> = (data) => {
    mutate({ ...data, treeIds: data.treeIds ?? [] })
  }

  const handleCancel = () => {
    navigate({ to: '/map', search: (prev) => prev }).catch((error) =>
      console.error('Navigation failed:', error),
    )
  }

  return (
    <>
      <MapPanel title={t('clusterForm.newTitle')} onClose={handleCancel}>
        <p className="mb-5 shrink-0 text-sm text-dark-600">{t('clusterForm.clickTreesHint')}</p>
        <FormProvider {...form}>
          <FormForTreecluster
            displayError={isError}
            errorMessage={error?.message}
            onSubmit={onSubmit}
            onBlur={saveDraft}
            fullWidth
            emptyHint={t('clusterForm.emptyHint')}
            organizations={organizations}
            onOrganizationChange={changeOrganization}
            discardedTreeCount={discardedTreeCount}
          />
        </FormProvider>
      </MapPanel>

      <ForeignTreeDialog
        open={foreignTree !== null}
        onOpenChange={(open) => !open && setForeignTree(null)}
        organizationName={foreignTree ? nameOf(foreignTree.organizationId) : undefined}
        canSwitch={foreignTree ? canCreateIn(foreignTree.organizationId) : false}
        blockedReason={t('clusterForm.blockedReasonCreate')}
        selectedTreeCount={treeIds.length}
        onConfirm={adoptForeignTree}
      />

      <UnsavedChangesDialog blocker={navigationBlocker} />
    </>
  )
}
