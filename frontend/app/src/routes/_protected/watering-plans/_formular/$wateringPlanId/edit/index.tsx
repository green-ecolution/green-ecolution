import WateringPlanUpdate from '@/components/watering-plan/WateringPlanUpdate'
import { pendingLoading } from '@/lib/router'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'
import { useWateringPlanDraft } from '@/store/form/useFormDraft'
import { WateringPlanForm } from '@/schema/wateringPlanSchema'

const wateringPlanFormRoute = getRouteApi('/_protected/watering-plans/_formular/$wateringPlanId')

export const Route = createFileRoute('/_protected/watering-plans/_formular/$wateringPlanId/edit/')({
  component: EditWateringPlan,
  pendingComponent: pendingLoading('Einsatzplan wird geladen …'),
})

function EditWateringPlan() {
  const { wateringPlan } = wateringPlanFormRoute.useLoaderData()
  const draft = useWateringPlanDraft<WateringPlanForm>('update')

  const formKey = draft.data?.clusterIds?.join(',') ?? 'initial'

  return (
    <div className="container mt-6">
      <WateringPlanUpdate key={formKey} wateringPlanId={wateringPlan.id.toString()} />
    </div>
  )
}
