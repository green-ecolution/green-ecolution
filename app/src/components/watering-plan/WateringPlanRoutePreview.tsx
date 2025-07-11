import ZoomControls from '@/components/map/ZoomControls'
import { Suspense } from 'react'
import LoadingInfo from '@/components/general/error/LoadingInfo'
import ShowRoutePreview from '../map/marker/ShowRoutePreview'
import { WithTreesAndClusters } from '../map/marker/WithAllClusterAndTrees'
import { Tree, TreeCluster, WateringPlan } from '@green-ecolution/backend-client'
import { useNavigate } from '@tanstack/react-router'
import Map from '../map/Map'

interface WateringPlanPreviewRouteProps {
  wateringPlan: WateringPlan
}

const WateringPlanPreviewRoute = ({ wateringPlan }: WateringPlanPreviewRouteProps) => {
  const navigate = useNavigate({})

  const handleTreeClick = (tree: Tree) => {
    navigate({ to: `/trees/$treeId`, params: { treeId: tree.id.toString() } }).catch((error) =>
      console.error('Navigation failed:', error),
    )
  }

  const handleClusterClick = (cluster: TreeCluster) => {
    navigate({
      to: `/treecluster/$treeclusterId`,
      params: { treeclusterId: cluster.id.toString() },
    }).catch((error) => console.error('Navigation failed:', error))
  }

  return (
    <Map width="100%" height="40rem">
      <ZoomControls />
      <Suspense fallback={<LoadingInfo label="Lade Karte..." />}>
        <ShowRoutePreview
          selectedClustersIds={wateringPlan.treeclusters.map((tc) => tc.id)}
          transporterId={wateringPlan.transporter.id}
          trailerId={wateringPlan.trailer?.id}
        />

        <WithTreesAndClusters onClickTree={handleTreeClick} onClickCluster={handleClusterClick} />
      </Suspense>
    </Map>
  )
}

export default WateringPlanPreviewRoute
