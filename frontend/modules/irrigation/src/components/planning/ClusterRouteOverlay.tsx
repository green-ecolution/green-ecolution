import { Suspense, useCallback, useState } from 'react'
import { useNavigate } from '@tanstack/react-router'
import type { PlanResponse } from '@green-ecolution/backend-client'
import { LAYERS as CORE_LAYERS } from '@/components/map-gl/mapStyle'
import useClusterBoundaryLayer from '../map-gl/layers/useClusterBoundaryLayer'
import useClusterMarkerLayer from '../map-gl/layers/useClusterMarkerLayer'
import ClusterPanel from '../map/cluster-panel/ClusterPanel'

const ClusterRouteOverlay = ({ plan }: { plan: PlanResponse }) => {
  const [selectedClusterId, setSelectedClusterId] = useState<string | null>(null)
  const navigate = useNavigate()

  const handleSelectCluster = useCallback((id: string) => setSelectedClusterId(id), [])

  // Boundaries stay below the route, markers above it.
  useClusterBoundaryLayer({
    onBoundaryClick: handleSelectCluster,
    clusterIds: plan.stops,
    beforeLayerId: CORE_LAYERS.routeCasing,
  })
  useClusterMarkerLayer({
    onClusterClick: handleSelectCluster,
    clusterIds: plan.stops,
    flyToOnClick: false,
  })

  if (!selectedClusterId) return null

  const goTo = (to: '/treecluster/$treeclusterId' | '/map/treecluster/edit/$treeclusterId') =>
    navigate({ to, params: { treeclusterId: selectedClusterId } }).catch((error) =>
      console.error('Navigation failed:', error),
    )

  return (
    <Suspense fallback={null}>
      <ClusterPanel
        key={selectedClusterId}
        clusterId={selectedClusterId}
        onClose={() => setSelectedClusterId(null)}
        onOpenDashboard={() => void goTo('/treecluster/$treeclusterId')}
        onEdit={() => void goTo('/map/treecluster/edit/$treeclusterId')}
      />
    </Suspense>
  )
}

export default ClusterRouteOverlay
