import LoadingInfo from '@/components/general/error/LoadingInfo'
import WateringPlanUpdate from '@/components/watering-plan/WateringPlanUpdate'
import useStore from '@/store/store'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/watering-plans/_formular/$wateringPlanId/edit/')({
  component: StatusEditWateringPlan,
  pendingComponent: () => <LoadingInfo label="Einsatzplan wird geladen …" />,
  loader: () => {
    if (!useStore.getState().isAuthenticated) return
  },
})

function StatusEditWateringPlan() {
  const wateringPlanId = Route.useParams().wateringPlanId

  return (
    <div className="container mt-6">
      <WateringPlanUpdate wateringPlanId={wateringPlanId} />
    </div>
  )
}
