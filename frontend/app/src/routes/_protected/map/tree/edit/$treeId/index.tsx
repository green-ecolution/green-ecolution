import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useCallback, useEffect, useRef, useState } from 'react'
import { FormProvider, type DefaultValues } from 'react-hook-form'
import { useSuspenseQuery } from '@tanstack/react-query'
import { Button } from '@green-ecolution/ui'
import { Trash2 } from 'lucide-react'
import { sensorQuery, treeClusterQuery, treeIdQuery } from '@/api/queries'
import { treeApi } from '@/api/backendApi'
import { TreeForm } from '@/schema/treeSchema'
import { useTreeForm } from '@/hooks/form/useTreeForm'
import createToast from '@/hooks/createToast'
import { entityNotFound, prefetch } from '@/lib/router'
import FormForTree from '@/components/general/form/FormForTree'
import DeleteConfirmDialog from '@/components/general/DeleteConfirmDialog'
import UnsavedChangesDialog from '@/components/general/form/UnsavedChangesDialog'
import DraggableMarker, { type DraggableMarkerLngLat } from '@/components/map-gl/DraggableMarker'
import MapPanel from '@/components/map-gl/MapPanel'
import { useMaplibreMap } from '@/components/map-gl/MapContext'
import { isMapAlive } from '@/components/map-gl/mapReady'
import useClusterBoundaryLayer from '@/components/map-gl/layers/useClusterBoundaryLayer'
import useClusterMarkerLayer from '@/components/map-gl/layers/useClusterMarkerLayer'
import useTreeLayers from '@/components/map-gl/layers/useTreeLayers'

export const Route = createFileRoute('/_protected/map/tree/edit/$treeId/')({
  component: EditTreeOnMap,
  loader: ({ context: { queryClient }, params: { treeId } }) => {
    prefetch(queryClient, treeIdQuery(treeId), 'treeIdQuery')
    prefetch(queryClient, sensorQuery(), 'sensorQuery')
    prefetch(queryClient, treeClusterQuery(), 'treeClusterQuery')
  },
  errorComponent: entityNotFound({
    entityName: 'Baum',
    backTo: '/trees',
    backLabel: 'Zur Baumliste',
  }),
})

function EditTreeOnMap() {
  const { treeId } = Route.useParams()
  const navigate = useNavigate({ from: Route.fullPath })
  const showToast = createToast()
  const map = useMaplibreMap()
  const { data: tree } = useSuspenseQuery(treeIdQuery(treeId))
  const { data: sensors } = useSuspenseQuery(sensorQuery())
  const { data: treeClusters } = useSuspenseQuery(treeClusterQuery())
  const [pos, setPos] = useState<DraggableMarkerLngLat>({ lng: tree.longitude, lat: tree.latitude })
  const [confirmDelete, setConfirmDelete] = useState(false)

  const isProvider = !!tree.provider

  const initForm: DefaultValues<TreeForm> = {
    latitude: tree.latitude,
    longitude: tree.longitude,
    number: tree.number,
    species: tree.species,
    plantingYear: tree.plantingYear,
    treeClusterId: tree.treeClusterId ?? null,
    sensorId: tree.sensor?.id ?? null,
    description: tree.description,
    provider: tree.provider ?? undefined,
  }

  const { mutate, isError, error, form, navigationBlocker, saveDraft } = useTreeForm('update', {
    treeId,
    initForm,
  })

  useClusterBoundaryLayer({ interactive: false })
  useClusterMarkerLayer({ interactive: false })
  useTreeLayers({ interactive: false })

  // Frame the tree once when the panel opens; later coordinate edits are driven
  // by the draggable marker, not this fly-to.
  const initialCoord = useRef({ lng: tree.longitude, lat: tree.latitude })
  useEffect(() => {
    if (!isMapAlive(map)) return
    map.flyTo({ center: [initialCoord.current.lng, initialCoord.current.lat], zoom: 18 })
  }, [map])

  const handleDragEnd = useCallback(
    (p: DraggableMarkerLngLat) => {
      setPos(p)
      form.setValue('latitude', p.lat, { shouldValidate: true, shouldDirty: true })
      form.setValue('longitude', p.lng, { shouldValidate: true, shouldDirty: true })
    },
    [form],
  )

  const onSubmit = (data: TreeForm) => {
    mutate({
      ...data,
      sensorId: data.sensorId ?? undefined,
      treeClusterId: data.treeClusterId ?? undefined,
    })
  }

  const handleCancel = () => {
    navigate({ to: '/map', search: (prev) => prev }).catch((error) =>
      console.error('Navigation failed:', error),
    )
  }

  const handleDelete = () => {
    navigationBlocker.allowNavigation()
    treeApi
      .deleteTree({ treeId })
      .then(() => navigate({ to: '/map', search: (prev) => prev }))
      .then(() => showToast('Der Baum wurde gelöscht.'))
      .catch((error) => {
        console.error('Delete failed:', error)
        showToast('Der Baum konnte nicht gelöscht werden.', 'error')
      })
  }

  return (
    <>
      {!isProvider && <DraggableMarker lng={pos.lng} lat={pos.lat} onDragEnd={handleDragEnd} />}

      <MapPanel title="Baum bearbeiten" onClose={handleCancel} className="overflow-y-auto">
        {!isProvider && (
          <p className="mb-5 shrink-0 text-sm text-dark-600">
            Ziehe den Marker auf der Karte, um den Standort des Baums anzupassen.
          </p>
        )}
        <FormProvider {...form}>
          <FormForTree
            isReadonly={isProvider}
            treeClusters={treeClusters.data}
            sensors={sensors.data}
            displayError={isError}
            errorMessage={error?.message}
            onSubmit={onSubmit}
            onBlur={saveDraft}
            hideLocation
            fullWidth
          />
        </FormProvider>
        {!isProvider && (
          <Button
            type="button"
            variant="ghost"
            onClick={() => setConfirmDelete(true)}
            className="mt-3 shrink-0 self-start text-destructive hover:text-destructive"
          >
            <Trash2 className="size-4" />
            Baum löschen
          </Button>
        )}
      </MapPanel>

      <DeleteConfirmDialog
        open={confirmDelete}
        onOpenChange={setConfirmDelete}
        title="Baum löschen?"
        description="Möchtest du diesen Baum wirklich löschen? Diese Aktion kann nicht rückgängig gemacht werden."
        onConfirm={handleDelete}
      />

      <UnsavedChangesDialog blocker={navigationBlocker} />
    </>
  )
}
