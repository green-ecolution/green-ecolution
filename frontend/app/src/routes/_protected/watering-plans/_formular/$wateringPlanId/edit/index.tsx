import { Loading } from '@green-ecolution/ui'
import WateringPlanUpdate from '@/components/watering-plan/WateringPlanUpdate'
import { createFileRoute, getRouteApi } from '@tanstack/react-router'
import { useWateringPlanDraft } from '@/store/form/useFormDraft'
import { WateringPlanForm } from '@/schema/wateringPlanSchema'

const wateringPlanFormRoute = getRouteApi('/_protected/watering-plans/_formular/$wateringPlanId')

export const Route = createFileRoute('/_protected/watering-plans/_formular/$wateringPlanId/edit/')({
  component: EditWateringPlan,
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Einsatzplan wird geladen …" />
  ),
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
