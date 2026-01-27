import { Loading } from '@green-ecolution/ui'
import WateringPlanDashboard from '@/components/watering-plan/WateringPlanDashboard'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'

const wateringPlanRoute = getRouteApi('/_protected/watering-plans/$wateringPlanId')

export const Route = createFileRoute('/_protected/watering-plans/$wateringPlanId/')({
  component: SingleWateringPlan,
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Einsatzplan wird geladen..." />
  ),
})

function SingleWateringPlan() {
  const { wateringPlan } = wateringPlanRoute.useLoaderData()

  return (
    <div className="container mt-6">
      <WateringPlanDashboard wateringPlan={wateringPlan} />
    </div>
  )
}

export default SingleWateringPlan
