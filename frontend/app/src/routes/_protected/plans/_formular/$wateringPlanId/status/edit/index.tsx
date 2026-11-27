import PlanStatusUpdate from '@/components/plan/PlanStatusUpdate'
import { getAuthSession } from '@/lib/auth/session'
import { pendingLoading } from '@/lib/router'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'

const planFormRoute = getRouteApi('/_protected/plans/_formular/$wateringPlanId')

export const Route = createFileRoute('/_protected/plans/_formular/$wateringPlanId/status/edit/')({
  component: StatusEditPlan,
  pendingComponent: pendingLoading({ key: 'wateringPlan:detail.loadingLabel' }),
  loader: async () => {
    if (!(await getAuthSession().isAuthenticated())) return
  },
})

function StatusEditPlan() {
  const { wateringPlan: plan } = planFormRoute.useLoaderData()

  return (
    <div className="container mt-6">
      <PlanStatusUpdate wateringPlanId={plan.id.toString()} />
    </div>
  )
}
