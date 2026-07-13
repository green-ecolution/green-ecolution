import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useCallback, useEffect, useRef } from 'react'
import { FormProvider, useWatch, type DefaultValues, type SubmitHandler } from 'react-hook-form'
import { useSuspenseQuery } from '@tanstack/react-query'
import type { TreeResponse } from '@green-ecolution/backend-client'
import { treeClusterIdQuery } from '@/api/queries'
import { TreeclusterForm } from '@/schema/treeclusterSchema'
import { entityNotFound } from '@/lib/router'
import FormForTreecluster from '@/components/general/form/FormForTreecluster'
import UnsavedChangesDialog from '@/components/general/form/UnsavedChangesDialog'
import { useTreeClusterForm } from '@/hooks/form/useTreeClusterForm'
import MapPanel from '@/components/map-gl/MapPanel'
import { useMaplibreMap } from '@/components/map-gl/MapContext'
import { isMapAlive } from '@/components/map-gl/mapReady'
import useClusterBoundaryLayer from '@/components/map-gl/layers/useClusterBoundaryLayer'
import useSelectableTreeLayer from '@/components/map-gl/layers/useSelectableTreeLayer'

export const Route = createFileRoute('/_protected/map/treecluster/edit/$treeclusterId/')({
  component: EditClusterOnMap,
  loader: ({ context: { queryClient }, params: { treeclusterId } }) =>
    queryClient.prefetchQuery(treeClusterIdQuery(treeclusterId)),
  errorComponent: entityNotFound({
    entityName: 'Bewässerungsgruppe',
    backTo: '/treecluster',
    backLabel: 'Zur Gruppenliste',
  }),
})

function EditClusterOnMap() {
  const { treeclusterId } = Route.useParams()
  const navigate = useNavigate({ from: Route.fullPath })
  const map = useMaplibreMap()
  const { data: cluster } = useSuspenseQuery(treeClusterIdQuery(treeclusterId))

  // Frame the group once when the panel opens.
  const initialCluster = useRef(cluster)
  useEffect(() => {
    if (!isMapAlive(map)) return
    const framed = initialCluster.current
    const trees = framed.trees ?? []
    if (trees.length > 0) {
      const lngs = trees.map((t) => t.longitude)
      const lats = trees.map((t) => t.latitude)
      map.fitBounds(
        [
          [Math.min(...lngs), Math.min(...lats)],
          [Math.max(...lngs), Math.max(...lats)],
        ],
        { padding: 60, maxZoom: 17 },
      )
    } else if (framed.longitude !== 0 || framed.latitude !== 0) {
      map.flyTo({ center: [framed.longitude, framed.latitude], zoom: 17 })
    }
  }, [map])

  const initForm: DefaultValues<TreeclusterForm> = {
    name: cluster.name,
    address: cluster.address,
    description: cluster.description,
    soilCondition: cluster.soilCondition,
    treeIds: cluster.trees?.map((tree: TreeResponse) => tree.id) ?? [],
  }

  const { mutate, isError, error, form, navigationBlocker, saveDraft } = useTreeClusterForm(
    'update',
    {
      clusterId: treeclusterId,
      initForm,
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
      <MapPanel title="Bewässerungsgruppe bearbeiten" onClose={handleCancel}>
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
