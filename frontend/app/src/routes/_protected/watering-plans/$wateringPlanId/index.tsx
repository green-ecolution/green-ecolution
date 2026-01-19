import { wateringPlanIdQuery } from '@/api/queries'
import { Loading } from '@green-ecolution/ui'
import WateringPlanDashboard from '@/components/watering-plan/WateringPlanDashboard'
import { useSuspenseQuery } from '@tanstack/react-query'
import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/watering-plans/$wateringPlanId/')({
  component: SingleWateringPlan,
  pendingComponent: () => <Loading className="mt-20 justify-center" label="Einsatzplan wird geladen..." />,
  loader: async ({ context: { queryClient }, params }) =>
    queryClient.prefetchQuery(wateringPlanIdQuery(params.wateringPlanId)),
})

function SingleWateringPlan() {
  const wateringPlanId = Route.useParams().wateringPlanId
  const { data: wateringPlan } = useSuspenseQuery(wateringPlanIdQuery(wateringPlanId))

  return (
    <div className="container mt-6">
      <WateringPlanDashboard wateringPlan={wateringPlan} />
    </div>
  )
}

export default WateringPlanDashboard
