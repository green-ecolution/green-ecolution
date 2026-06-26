import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useCallback } from 'react'
import { FormProvider, useWatch, type DefaultValues, type SubmitHandler } from 'react-hook-form'
import { SoilCondition } from '@/api/backendApi'
import { TreeclusterForm } from '@/schema/treeclusterSchema'
import FormForTreecluster from '@/components/general/form/FormForTreecluster'
import UnsavedChangesDialog from '@/components/general/form/UnsavedChangesDialog'
import { useTreeClusterForm } from '@/hooks/form/useTreeClusterForm'
import MapPanel from '@/components/map-gl/MapPanel'
import useClusterBoundaryLayer from '@/components/map-gl/layers/useClusterBoundaryLayer'
import useSelectableTreeLayer from '@/components/map-gl/layers/useSelectableTreeLayer'

export const Route = createFileRoute('/_protected/map/treecluster/new/')({
  component: NewClusterOnMap,
})

const defaultForm: DefaultValues<TreeclusterForm> = {
  soilCondition: SoilCondition.Unknown,
  treeIds: [],
}

function NewClusterOnMap() {
  const navigate = useNavigate({ from: Route.fullPath })
  const { mutate, isError, error, form, navigationBlocker, saveDraft } = useTreeClusterForm(
    'create',
    {
      initForm: defaultForm,
    },
  )
  const treeIds = useWatch({ control: form.control, name: 'treeIds' }) ?? []

  const toggleTree = useCallback(
    (id: string) => {
      const current = form.getValues('treeIds') ?? []
      const next = current.includes(id) ? current.filter((x) => x !== id) : [...current, id]
      form.setValue('treeIds', next, { shouldValidate: true, shouldDirty: true })
    },
    [form],
  )

  useClusterBoundaryLayer({ interactive: false })
  useSelectableTreeLayer({ selectedIds: treeIds, onToggle: toggleTree })

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
      <MapPanel title="Neue Bewässerungsgruppe" onClose={handleCancel}>
        <p className="mb-5 shrink-0 text-sm text-dark-600">
          Klicke Bäume auf der Karte an, um sie der Gruppe hinzuzufügen oder zu entfernen.
        </p>
        <FormProvider {...form}>
          <FormForTreecluster
            displayError={isError}
            errorMessage={error?.message}
            onSubmit={onSubmit}
            onBlur={saveDraft}
            fullWidth
            emptyHint="Klicke einen Baum auf der Karte an, um ihn zur Liste hinzuzufügen."
          />
        </FormProvider>
      </MapPanel>

      <UnsavedChangesDialog blocker={navigationBlocker} />
    </>
  )
}
