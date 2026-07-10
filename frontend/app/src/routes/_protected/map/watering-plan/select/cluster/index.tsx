import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { useCallback, useEffect, useMemo, useState } from 'react'
import { Button, InlineAlert } from '@green-ecolution/ui'
import { MoveRight } from 'lucide-react'
import { useQuery, useSuspenseQuery } from '@tanstack/react-query'
import { z } from 'zod'
import { WateringPlanForm } from '@/schema/wateringPlanSchema'
import {
  routePreviewQuery,
  routingStartPointsQuery,
  treeClusterQuery,
  vehicleIdQuery,
} from '@/api/queries'
import { useWateringPlanDraft } from '@/store/form/useFormDraft'
import { useFormNavigationBlocker } from '@/hooks/form/useFormNavigationBlocker'
import SelectEntities from '@/components/general/form/types/SelectEntities'
import UnsavedChangesDialog from '@/components/general/form/UnsavedChangesDialog'
import MapPanel from '@/components/map-gl/MapPanel'
import useSelectableClusterLayer from '@/components/map-gl/layers/useSelectableClusterLayer'
import useRouteLayer from '@/components/map-gl/layers/useRouteLayer'
import RoutePointMarkers, {
  buildRoutePoints,
  type RoutePointMarkerData,
} from '@/components/map-gl/RoutePointMarkers'

const mapSelectClusterSchema = z.object({
  transporterId: z.string().optional(),
  trailerId: z.string().optional(),
  formType: z.enum(['create', 'update']),
  clusterIds: z.array(z.string()),
  wateringPlanId: z.string().optional(),
})

export const Route = createFileRoute('/_protected/map/watering-plan/select/cluster/')({
  component: SelectCluster,
  validateSearch: mapSelectClusterSchema,
  loader: ({ context: { queryClient } }) => queryClient.prefetchQuery(treeClusterQuery()),
})

function SelectCluster() {
  const { trailerId, transporterId, formType, clusterIds: searchClusterIds } = Route.useSearch()
  const [clusterIds, setClusterIds] = useState<string[]>(searchClusterIds)
  const [showError, setShowError] = useState(false)
  const navigate = useNavigate({ from: Route.fullPath })
  const { wateringPlanId } = Route.useSearch()
  const draft = useWateringPlanDraft<WateringPlanForm>(formType)

  const navigationBlocker = useFormNavigationBlocker({
    isDirty: true,
    allowedPaths: ['/watering-plans/', '/map/watering-plan/select/cluster'],
    onLeave: () => draft.clear(),
    message:
      'Möchtest du die Seite wirklich verlassen? Deine Eingaben gehen verloren, wenn du jetzt gehst.',
  })

  const { data: clusters } = useSuspenseQuery(treeClusterQuery())
  const { data: transporter } = useQuery({
    ...vehicleIdQuery(transporterId?.toString() ?? '-1'),
    enabled: !!transporterId && transporterId !== '-1',
  })
  const { data: trailer } = useQuery({
    ...vehicleIdQuery(trailerId?.toString() ?? '-1'),
    enabled: !!trailerId && trailerId !== '-1',
  })

  const [debouncedClusterIds, setDebouncedClusterIds] = useState<string[]>(clusterIds)
  useEffect(() => {
    const id = setTimeout(() => setDebouncedClusterIds(clusterIds), 600)
    return () => clearTimeout(id)
  }, [clusterIds])

  const previewEnabled = debouncedClusterIds.length > 0 && !!transporterId && transporterId !== '-1'
  const { data: previewRoute } = useQuery({
    ...routePreviewQuery(debouncedClusterIds, transporterId ?? '', draft.data?.startPointName),
    enabled: previewEnabled,
  })
  const routeCoordinates = previewEnabled
    ? (previewRoute?.geometry?.coordinates as [number, number][] | undefined)
    : undefined
  useRouteLayer({ coordinates: routeCoordinates })

  const { data: startPoints } = useQuery(routingStartPointsQuery())
  const startPointName = draft.data?.startPointName
  const routePoints = useMemo<RoutePointMarkerData[]>(
    () =>
      buildRoutePoints(
        startPoints,
        startPointName,
        previewEnabled ? (previewRoute?.refillPoints ?? undefined) : undefined,
      ),
    [startPoints, startPointName, previewRoute, previewEnabled],
  )

  const { allowNavigation } = navigationBlocker

  const handleNavigateBack = useCallback(() => {
    allowNavigation()
    switch (formType) {
      case 'update':
        return navigate({
          to: `/watering-plans/$wateringPlanId/edit`,
          params: { wateringPlanId: String(wateringPlanId) },
        })
      case 'create':
        return navigate({ to: '/watering-plans/new' })
    }
  }, [navigate, formType, wateringPlanId, allowNavigation])

  const disabledClusters = useMemo(() => {
    if (!transporter) return clusters.data.map((cluster) => cluster.id)

    const totalCapacity = trailer
      ? transporter.waterCapacity + trailer.waterCapacity
      : transporter.waterCapacity

    return clusters.data
      .filter((cluster) => {
        const neededWater = (cluster.treeIds?.length ?? 0) * 80
        return neededWater > totalCapacity
      })
      .map((cluster) => cluster.id)
  }, [transporter, trailer, clusters.data])

  const handleToggle = useCallback((id: string) => {
    setShowError(false)
    setClusterIds((prev) => (prev.includes(id) ? prev.filter((x) => x !== id) : [...prev, id]))
  }, [])

  useSelectableClusterLayer({
    selectedIds: clusterIds,
    disabledIds: disabledClusters,
    onToggle: handleToggle,
  })

  const handleSave = () => {
    if (clusterIds.length === 0) {
      setShowError(true)
      return
    }

    const originalClusterIds = draft.data?.clusterIds ?? searchClusterIds
    const clustersChanged =
      clusterIds.length !== originalClusterIds.length ||
      clusterIds.some((id) => !originalClusterIds.includes(id))

    draft.updateData((prev) => ({
      ...(prev ?? ({} as WateringPlanForm)),
      clusterIds,
    }))

    if (clustersChanged) {
      draft.markChanged()
    }

    handleNavigateBack()?.catch((error) => console.error('Navigation failed:', error))
  }

  const { showNotice, notice } = useMemo(() => {
    const errors = []

    if (!transporterId || transporterId === '-1') {
      errors.push('Um eine Route generieren zu können, muss ein Fahrzeug ausgewählt werden.')
    }

    if (disabledClusters.length > 0) {
      errors.push(
        'Ausgegraute Bewässerungsgruppen sind ausgeschlossen, da das Fahrzeug nicht genügend Wasserkapazität hat.',
      )
    }

    return { showNotice: errors.length > 0, notice: errors }
  }, [transporterId, disabledClusters])

  return (
    <>
      <RoutePointMarkers points={routePoints} />
      <MapPanel title="Bewässerungsgruppen auswählen" onClose={() => void handleNavigateBack()}>
        <p className="mb-5 shrink-0 text-sm text-dark-600">
          Klicke die Gruppen auf der Karte an, die in diesen Bewässerungsplan aufgenommen werden
          sollen.
        </p>
        {showNotice && <InlineAlert className="mb-4 shrink-0" description={notice.join(' ')} />}

        <SelectEntities
          onChange={setClusterIds}
          entityIds={clusterIds}
          type="cluster"
          label="Bewässerungsgruppen"
          fill
          emptyHint="Klicke eine Gruppe auf der Karte an, um sie hinzuzufügen."
        />
        {showError && clusterIds.length === 0 && (
          <p className="mt-2 shrink-0 text-sm font-semibold text-destructive">
            Bitte wähle mindestens eine Bewässerungsgruppe aus.
          </p>
        )}

        <Button
          type="button"
          onClick={handleSave}
          disabled={clusterIds.length === 0}
          className="mt-4 w-full shrink-0"
        >
          Übernehmen
          <MoveRight className="icon-arrow-animate" />
        </Button>
      </MapPanel>

      <UnsavedChangesDialog blocker={navigationBlocker} />
    </>
  )
}
