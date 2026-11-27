import PlanDashboard from '@/components/plan/PlanDashboard'
import { pendingLoading } from '@/lib/router'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'

const planRoute = getRouteApi('/_protected/plans/$wateringPlanId')

export const Route = createFileRoute('/_protected/plans/$wateringPlanId/')({
  component: SinglePlan,
  pendingComponent: pendingLoading({ key: 'wateringPlan:detail.loadingLabel' }),
})

function SinglePlan() {
  const { wateringPlan: plan } = planRoute.useLoaderData()

  return (
    <div className="container mt-6">
      <PlanDashboard plan={plan} />
    </div>
  )
}
