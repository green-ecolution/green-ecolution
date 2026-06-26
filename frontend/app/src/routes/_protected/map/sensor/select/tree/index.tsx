import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useCallback, useEffect, useState } from 'react'
import { useMutation, useQueryClient, useSuspenseQuery } from '@tanstack/react-query'
import { Button } from '@green-ecolution/ui'
import { MoveRight, X } from 'lucide-react'
import { z } from 'zod'
import { treeApi, type TreeUpdateRequest } from '@/api/backendApi'
import { sensorIdQuery, treeIdQuery } from '@/api/queries'
import SelectedCard from '@/components/general/cards/SelectedCard'
import createToast from '@/hooks/createToast'
import { useMaplibreMap } from '@/components/map-gl/MapContext'
import SensorMarker from '@/components/map-gl/SensorMarker'
import useSelectableTreeLayer from '@/components/map-gl/layers/useSelectableTreeLayer'
import { isMapAlive } from '@/components/map-gl/mapReady'

const searchSchema = z.object({
  sensorId: z.string(),
  lat: z.number().optional(),
  lng: z.number().optional(),
  zoom: z.number().optional(),
})

export const Route = createFileRoute('/_protected/map/sensor/select/tree/')({
  component: LinkTreeToSensor,
  validateSearch: searchSchema,
  loaderDeps: ({ search: { sensorId } }) => ({ sensorId }),
  loader: ({ context: { queryClient }, deps: { sensorId } }) =>
    queryClient.prefetchQuery(sensorIdQuery(String(sensorId))),
})

function LinkTreeToSensor() {
  const { sensorId } = Route.useSearch()
  const navigate = useNavigate({ from: Route.fullPath })
  const queryClient = useQueryClient()
  const showToast = createToast()
  const map = useMaplibreMap()
  const { data: sensor } = useSuspenseQuery(sensorIdQuery(String(sensorId)))
  const [treeId, setTreeId] = useState<string | undefined>(
    sensor.linkedTreeId != null ? String(sensor.linkedTreeId) : undefined,
  )
  const [showError, setShowError] = useState(false)

  const toggleTree = useCallback((id: string) => {
    setShowError(false)
    setTreeId((prev) => (prev === id ? undefined : id))
  }, [])

  useSelectableTreeLayer({ selectedIds: treeId ? [treeId] : [], onToggle: toggleTree })

  const lng = sensor.coordinate?.longitude
  const lat = sensor.coordinate?.latitude
  useEffect(() => {
    if (!isMapAlive(map) || lng == null || lat == null) return
    map.flyTo({ center: [lng, lat], zoom: 18 })
  }, [map, lng, lat])

  const handleNavigateBack = useCallback(
    () => navigate({ to: '/sensors/$sensorId', params: { sensorId: String(sensorId) } }),
    [navigate, sensorId],
  )

  const { mutate } = useMutation({
    mutationFn: (body: TreeUpdateRequest) =>
      treeApi.updateTree({ treeId: treeId!, treeUpdateRequest: body }),
    onSuccess: () => {
      queryClient
        .invalidateQueries(sensorIdQuery(String(sensorId)))
        .catch((error) => console.error('Invalidate failed:', error))
      handleNavigateBack()
        .then(() => showToast('Vegetation wurde erfolgreich verlinkt'))
        .catch((error) => console.error('Navigation failed:', error))
    },
    onError: (error: Error) => {
      console.error('Error linking sensor to tree:', error)
      showToast('Die Verknüpfung ist fehlgeschlagen. Bitte später erneut versuchen.', 'error')
    },
  })

  const handleSave = async () => {
    if (!treeId) {
      setShowError(true)
      return
    }
    try {
      const data = await queryClient.fetchQuery(treeIdQuery(String(treeId)))
      mutate({ ...data, sensorId })
    } catch (error) {
      console.error('Error fetching tree data:', error)
      showToast('Die Baumdaten konnten nicht geladen werden.', 'error')
    }
  }

  return (
    <>
      {lng != null && lat != null && <SensorMarker lng={lng} lat={lat} />}
      <div className="absolute top-4 right-4 z-[1030] flex max-h-[calc(100%-2rem)] w-[26rem] max-w-[calc(100%-2rem)] flex-col rounded-xl bg-white p-5 font-nunito-sans shadow-xl">
        <div className="mb-4 flex shrink-0 items-center justify-between gap-4">
          <h2 className="font-lato text-lg font-semibold">Baum verknüpfen</h2>
          <Button
            variant="ghost"
            size="icon"
            aria-label="Abbrechen"
            onClick={() => void handleNavigateBack()}
          >
            <X />
          </Button>
        </div>
        <p className="mb-5 shrink-0 text-sm text-dark-600">
          Klicke den Baum auf der Karte an, mit dem dieser Sensor verknüpft werden soll.
        </p>

        <div className="min-h-0 flex-1 overflow-y-auto">
          {treeId ? (
            <SelectedCard type="tree" id={treeId} onClick={() => setTreeId(undefined)} />
          ) : (
            <div className="rounded-lg border border-dashed border-dark-200 bg-dark-50/60 px-4 py-6 text-center text-sm">
              <p className="font-medium text-dark-800">Noch kein Baum ausgewählt.</p>
              <p className="mt-1 text-dark-600">
                Klicke einen Baum auf der Karte an, um ihn zu verknüpfen.
              </p>
            </div>
          )}
          {showError && (
            <p className="mt-2 text-sm font-semibold text-red">
              Bitte wähle einen Baum auf der Karte aus.
            </p>
          )}
        </div>

        <Button
          type="button"
          disabled={!treeId}
          onClick={() => void handleSave()}
          className="mt-4 w-full shrink-0"
        >
          Verknüpfen
          <MoveRight className="icon-arrow-animate" />
        </Button>
      </div>
    </>
  )
}
