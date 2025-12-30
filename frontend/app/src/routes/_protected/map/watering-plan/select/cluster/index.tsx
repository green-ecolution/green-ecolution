import { createFileRoute, useNavigate } from '@tanstack/react-router'
import { TreeCluster } from '@green-ecolution/backend-client'
import { useCallback, useMemo, useState } from 'react'
import { wateringPlanSchemaBase } from '@/schema/wateringPlanSchema'
import MapSelectEntitiesModal from '@/components/map/MapSelectEntitiesModal'
import WithAllClusters from '@/components/map/marker/WithAllClusters'
import ShowRoutePreview from '@/components/map/marker/ShowRoutePreview'
import { useQuery, useSuspenseQuery } from '@tanstack/react-query'
import { treeClusterQuery, vehicleIdQuery } from '@/api/queries'
import Notice from '@/components/general/Notice'
import SelectedCard from '@/components/general/cards/SelectedCard'
import { z } from 'zod'
import { safeJsonStorageParse } from '@/lib/utils'

const mapSelectClusterSchema = z.object({
  transporterId: z.coerce.number().optional(),
  trailerId: z.coerce.number().optional(),
  formType: z.enum(['create', 'update']),
  clusterIds: z.array(z.number().int()),
  wateringPlanId: z.number().optional(),
})

export const Route = createFileRoute('/_protected/map/watering-plan/select/cluster/')({
  component: SelectCluster,
  validateSearch: mapSelectClusterSchema,
  loader: ({ context: { queryClient } }) => {
    return queryClient.prefetchQuery(treeClusterQuery())
  },
})

function SelectCluster() {
  const { trailerId, transporterId, formType, clusterIds: searchClusterIds } = Route.useSearch()
  const [clusterIds, setClusterIds] = useState<number[]>(searchClusterIds)
  const [showError, setShowError] = useState(false)
  const navigate = useNavigate({ from: Route.fullPath })
  const { wateringPlanId } = Route.useSearch()
  const { data: clusters } = useSuspenseQuery(treeClusterQuery())
  const { data: transporter } = useQuery({
    ...vehicleIdQuery(transporterId?.toString() ?? '-1'),
    enabled: !!transporterId && transporterId !== -1,
  })
  const { data: trailer } = useQuery({
    ...vehicleIdQuery(trailerId?.toString() ?? '-1'),
    enabled: !!trailerId && trailerId !== -1,
  })

  const handleNavigateBack = useCallback(() => {
    switch (formType) {
      case 'update':
        return navigate({
          to: `/watering-plans/$wateringPlanId/edit`,
          params: { wateringPlanId: String(wateringPlanId) },
        })
      case 'create':
        return navigate({
          to: '/watering-plans/new',
        })
    }
  }, [navigate, formType, wateringPlanId])

  const handleSave = () => {
    if (clusterIds.length === 0) {
      setShowError(true)
      return
    }
    const { success, data, error } = safeJsonStorageParse(`${formType}-wateringplan`, {
      schema: wateringPlanSchemaBase,
    })

    if (success) {
      data.clusterIds = clusterIds
      window.sessionStorage.setItem(`${formType}-wateringplan`, JSON.stringify(data))
    } else {
      console.error(error)
    }

    handleNavigateBack().catch((error) => console.error('Navigation failed:', error))
  }

  const handleDelete = (clusterId: number) => {
    setClusterIds((prev) => prev.filter((id) => id !== clusterId))
  }

  const handleClick = (cluster: TreeCluster) => {
    if (disabledClusters.includes(cluster.id)) return

    if (clusterIds.includes(cluster.id))
      setClusterIds((prev) => prev.filter((id) => id !== cluster.id))
    else setClusterIds((prev) => [...prev, cluster.id])
  }

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

  const { showNotice, notice } = useMemo(() => {
    const errors = []

    if (!transporterId || transporterId === -1) {
      errors.push('Um eine Route generieren zu können, muss ein Fahrzeug ausgewählt werden.')
    }

    if (disabledClusters.length > 0) {
      errors.push(
        'Ausgegraute Bewässerungsgruppen sind ausgeschlossen, da das Fahrzeug nicht genügend Wasserkapazität hat.',
      )
    }

    return {
      showNotice: errors.length > 0,
      notice: errors,
    }
  }, [transporterId, disabledClusters])

  return (
    <>
      <MapSelectEntitiesModal
        onSave={handleSave}
        onCancel={() => void handleNavigateBack()}
        disabled={clusterIds.length === 0}
        title="Ausgewählte Bewässerungsgruppen:"
        content={
          <ul>
            {showNotice && <Notice classes="mb-4" description={notice.join(' ')} />}
            {(clusterIds?.length || 0) === 0 || showError ? (
              <li className="text-dark-600 font-semibold text-sm">
                <p>Hier können Sie zugehörigen Gruppen verlinken.</p>
              </li>
            ) : (
              clusterIds.map((clusterId) => (
                <li key={clusterId}>
                  <SelectedCard type="cluster" id={clusterId} onClick={handleDelete} />
                </li>
              ))
            )}
          </ul>
        }
      />
      <WithAllClusters
        onClick={handleClick}
        highlightedClusters={clusterIds}
        disabledClusters={disabledClusters}
      />

      {clusterIds.length > 0 && transporterId && transporterId != -1 && (
        <ShowRoutePreview
          selectedClustersIds={clusterIds}
          transporterId={transporterId}
          trailerId={trailerId}
        />
      )}
    </>
  )
}
