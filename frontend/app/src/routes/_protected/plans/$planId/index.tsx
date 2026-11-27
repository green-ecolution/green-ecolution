import PlanDashboard from '@/components/plan/PlanDashboard'
import { pendingLoading } from '@/lib/router'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'

const planRoute = getRouteApi('/_protected/plans/$planId')

export const Route = createFileRoute('/_protected/plans/$planId/')({
  component: SinglePlan,
  pendingComponent: pendingLoading({ key: 'plan:detail.loadingLabel' }),
})

function SinglePlan() {
  const { plan } = planRoute.useLoaderData()

  return (
    <div className="container mt-6">
      <PlanDashboard plan={plan} />
    </div>
  )
}
