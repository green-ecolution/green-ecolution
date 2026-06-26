import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useCallback, useEffect, useState } from 'react'
import { FormProvider, type DefaultValues } from 'react-hook-form'
import { useSuspenseQuery } from '@tanstack/react-query'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  Button,
} from '@green-ecolution/ui'
import { MoveRight, Trash2, X } from 'lucide-react'
import { sensorQuery, treeClusterQuery, treeIdQuery } from '@/api/queries'
import { treeApi } from '@/api/backendApi'
import { TreeForm } from '@/schema/treeSchema'
import { useTreeForm } from '@/hooks/form/useTreeForm'
import createToast from '@/hooks/createToast'
import FormForTree from '@/components/general/form/FormForTree'
import DraggableMarker, { type DraggableMarkerLngLat } from '@/components/map-gl/DraggableMarker'
import { useMaplibreMap } from '@/components/map-gl/MapContext'
import { isMapAlive } from '@/components/map-gl/mapReady'
import useClusterBoundaryLayer from '@/components/map-gl/layers/useClusterBoundaryLayer'
import useClusterMarkerLayer from '@/components/map-gl/layers/useClusterMarkerLayer'
import useTreeLayers from '@/components/map-gl/layers/useTreeLayers'

export const Route = createFileRoute('/_protected/map/tree/edit/$treeId/')({
  component: EditTreeOnMap,
  loader: ({ context: { queryClient }, params: { treeId } }) => {
    queryClient
      .prefetchQuery(treeIdQuery(treeId))
      .catch((error) => console.error('Prefetching "treeIdQuery" failed:', error))
    queryClient
      .prefetchQuery(sensorQuery())
      .catch((error) => console.error('Prefetching "sensorQuery" failed:', error))
    queryClient
      .prefetchQuery(treeClusterQuery())
      .catch((error) => console.error('Prefetching "treeClusterQuery" failed:', error))
  },
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

  const { mutate, isError, error, form, saveDraft } = useTreeForm('update', {
    treeId,
    initForm,
    disableNavigationBlock: true,
  })

  useClusterBoundaryLayer({ interactive: false })
  useClusterMarkerLayer({ interactive: false })
  useTreeLayers({ interactive: false })

  useEffect(() => {
    if (!isMapAlive(map)) return
    map.flyTo({ center: [tree.longitude, tree.latitude], zoom: 18 })
    // Frame the tree once when the panel opens.
    // eslint-disable-next-line react-hooks/exhaustive-deps
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
    treeApi
      .deleteTree({ treeId })
      .then(() => navigate({ to: '/trees', search: { page: 1 } }))
      .then(() => showToast('Der Baum wurde gelöscht.'))
      .catch((error) => {
        console.error('Delete failed:', error)
        showToast('Der Baum konnte nicht gelöscht werden.', 'error')
      })
  }

  return (
    <>
      {!isProvider && <DraggableMarker lng={pos.lng} lat={pos.lat} onDragEnd={handleDragEnd} />}

      <div className="absolute top-4 right-4 z-[1030] flex max-h-[calc(100%-2rem)] w-[30rem] max-w-[calc(100%-2rem)] flex-col overflow-y-auto rounded-xl bg-white p-5 font-nunito-sans shadow-xl">
        <div className="mb-4 flex shrink-0 items-center justify-between gap-4">
          <h2 className="font-lato text-lg font-semibold">Baum bearbeiten</h2>
          <Button variant="ghost" size="icon" aria-label="Abbrechen" onClick={handleCancel}>
            <X />
          </Button>
        </div>
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
            className="mt-3 shrink-0 self-start text-red hover:text-red"
          >
            <Trash2 className="size-4" />
            Baum löschen
          </Button>
        )}
      </div>

      <AlertDialog open={confirmDelete} onOpenChange={setConfirmDelete}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Baum löschen?</AlertDialogTitle>
            <AlertDialogDescription>
              Möchtest du diesen Baum wirklich löschen? Diese Aktion kann nicht rückgängig gemacht
              werden.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>
              Abbrechen
              <X />
            </AlertDialogCancel>
            <AlertDialogAction onClick={handleDelete}>
              Löschen
              <MoveRight className="icon-arrow-animate" />
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  )
}
