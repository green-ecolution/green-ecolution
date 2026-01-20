import { Loading } from '@green-ecolution/ui'
import WateringPlanUpdate from '@/components/watering-plan/WateringPlanUpdate'
import { createFileRoute } from '@tanstack/react-router'
import { useWateringPlanDraft } from '@/store/form/useFormDraft'
import { WateringPlanForm } from '@/schema/wateringPlanSchema'

export const Route = createFileRoute('/_protected/watering-plans/_formular/$wateringPlanId/edit/')({
  component: StatusEditWateringPlan,
  pendingComponent: () => (
    <Loading className="mt-20 justify-center" label="Einsatzplan wird geladen …" />
  ),
})

function StatusEditWateringPlan() {
  const wateringPlanId = Route.useParams().wateringPlanId
  const draft = useWateringPlanDraft<WateringPlanForm>('update')

  const formKey = draft.data?.clusterIds?.join(',') ?? 'initial'

  return (
    <div className="container mt-6">
      <WateringPlanUpdate key={formKey} wateringPlanId={wateringPlanId} />
    </div>
  )
}
