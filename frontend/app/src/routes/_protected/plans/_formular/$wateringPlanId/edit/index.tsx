import PlanUpdate from '@/components/plan/PlanUpdate'
import { pendingLoading } from '@/lib/router'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'
import { usePlanDraft } from '@/store/form/useFormDraft'
import { PlanForm } from '@/schema/planSchema'

const planFormRoute = getRouteApi('/_protected/plans/_formular/$wateringPlanId')

export const Route = createFileRoute('/_protected/plans/_formular/$wateringPlanId/edit/')({
  component: EditPlan,
  pendingComponent: pendingLoading({ key: 'wateringPlan:detail.loadingLabel' }),
})

function EditPlan() {
  const { wateringPlan: plan } = planFormRoute.useLoaderData()
  const draft = usePlanDraft<PlanForm>('update')

  const formKey = draft.data?.stopIds?.join(',') ?? 'initial'

  return (
    <div className="container mt-6">
      <PlanUpdate key={formKey} wateringPlanId={plan.id.toString()} />
    </div>
  )
}
