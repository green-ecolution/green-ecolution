import PlanStatusUpdate from '@/components/plan/PlanStatusUpdate'
import { getAuthSession } from '@/lib/auth/session'
import { pendingLoading } from '@/lib/router'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'

const planFormRoute = getRouteApi('/_protected/plans/_formular/$planId')

export const Route = createFileRoute('/_protected/plans/_formular/$planId/status/edit/')({
  component: StatusEditPlan,
  pendingComponent: pendingLoading({ key: 'plan:detail.loadingLabel' }),
  loader: async () => {
    if (!(await getAuthSession().isAuthenticated())) return
  },
})

function StatusEditPlan() {
  const { plan } = planFormRoute.useLoaderData()

  return (
    <div className="container mt-6">
      <PlanStatusUpdate planId={plan.id.toString()} />
    </div>
  )
}
