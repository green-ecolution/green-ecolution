import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { FormProvider } from 'react-hook-form'
import { useSuspenseQuery } from '@tanstack/react-query'
import { MapPin } from 'lucide-react'
import { sensorQuery, treeClusterQuery } from '@/api/queries'
import { prefetch } from '@/lib/router'
import { useTreeForm } from '@/hooks/form/useTreeForm'
import { TreeForm } from '@/schema/treeSchema'
import FormForTree from '@/components/general/form/FormForTree'
import UnsavedChangesDialog from '@/components/general/form/UnsavedChangesDialog'
import { useMapClick, type MapClickLngLat } from '@/components/map-gl/useMapClick'
import DraggableMarker from '@/components/map-gl/DraggableMarker'
import MapPanel from '@/components/map-gl/MapPanel'
import useClusterBoundaryLayer from '@/components/map-gl/layers/useClusterBoundaryLayer'
import useClusterMarkerLayer from '@/components/map-gl/layers/useClusterMarkerLayer'
import useTreeLayers from '@/components/map-gl/layers/useTreeLayers'

export const Route = createFileRoute('/_protected/map/tree/new/')({
  component: NewTree,
  loader: ({ context: { queryClient } }) => {
    prefetch(queryClient, sensorQuery(), 'sensorQuery')
    prefetch(queryClient, treeClusterQuery(), 'treeClusterQuery')
  },
})

const defaultForm = () => ({
  plantingYear: new Date().getFullYear(),
  treeClusterId: null,
  sensorId: null,
})

function NewTree() {
  const navigate = useNavigate({ from: Route.fullPath })
  const [pos, setPos] = useState<MapClickLngLat>()
  const { data: sensors } = useSuspenseQuery(sensorQuery())
  const { data: treeClusters } = useSuspenseQuery(treeClusterQuery())

  useClusterBoundaryLayer({ interactive: false })
  useClusterMarkerLayer({ interactive: false })
  useTreeLayers({ interactive: false })

  // Only the first map click sets the location; afterwards the tree is moved by
  // dragging its marker, so a stray background click must not relocate it.
  useMapClick((p) => setPos((prev) => prev ?? p))

  const { mutate, isError, error, form, navigationBlocker, saveDraft } = useTreeForm('create', {
    initForm: defaultForm(),
  })

  useEffect(() => {
    if (!pos) return
    form.setValue('latitude', pos.lat, { shouldValidate: true, shouldDirty: true })
    form.setValue('longitude', pos.lng, { shouldValidate: true, shouldDirty: true })
  }, [pos, form])

  const onSubmit = (data: TreeForm) => {
    mutate({
      ...data,
      sensorId: data.sensorId ?? undefined,
      treeClusterId: data.treeClusterId ?? null,
    })
  }

  const handleCancel = () => {
    navigate({ to: '/map', search: (prev) => prev }).catch((error) =>
      console.error('Navigation failed:', error),
    )
  }

  return (
    <>
      <MapPanel
        title="Neuen Baum erfassen"
        onClose={handleCancel}
        className="overflow-y-auto"
        mobileCollapsedSnap="260px"
      >
        {pos ? (
          <>
            <div className="mb-5 flex items-center gap-3 rounded-lg bg-dark-50 px-3 py-2.5">
              <span className="flex size-7 shrink-0 items-center justify-center rounded-full bg-white text-primary-500 shadow-sm">
                <MapPin className="size-4" />
              </span>
              <div className="min-w-0">
                <p className="text-sm font-medium text-dark-800">Standort gewählt</p>
                <p className="truncate text-xs tabular-nums text-dark-400">
                  {pos.lat.toFixed(5)}, {pos.lng.toFixed(5)}
                </p>
              </div>
            </div>
            <FormProvider {...form}>
              <FormForTree
                isReadonly={false}
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
          </>
        ) : (
          <p className="text-dark-600">
            Klicke auf die Karte, um den Standort des neuen Baums zu setzen. Den Marker kannst du
            danach noch verschieben.
          </p>
        )}
      </MapPanel>

      {pos && <DraggableMarker lng={pos.lng} lat={pos.lat} onDragEnd={setPos} />}

      <UnsavedChangesDialog blocker={navigationBlocker} />
    </>
  )
}
