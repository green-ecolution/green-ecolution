import { Loading } from '@green-ecolution/ui'
import WateringPlanStatusUpdate from '@/components/watering-plan/WateringPlanStatusUpdate'
import { getAuthSession } from '@/lib/auth/session'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'

const wateringPlanFormRoute = getRouteApi('/_protected/watering-plans/_formular/$wateringPlanId')

export const Route = createFileRoute(
  '/_protected/watering-plans/_formular/$wateringPlanId/status/edit/',
)({
  component: StatusEditWateringPlan,
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Einsatzplan wird geladen …" />
  ),
  loader: async () => {
    if (!(await getAuthSession().isAuthenticated())) return
  },
})

function StatusEditWateringPlan() {
  const { wateringPlan } = wateringPlanFormRoute.useLoaderData()

  return (
    <div className="container mt-6">
      <WateringPlanStatusUpdate wateringPlanId={wateringPlan.id.toString()} />
    </div>
  )
}
