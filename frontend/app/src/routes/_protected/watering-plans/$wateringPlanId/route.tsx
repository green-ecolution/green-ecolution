import { createFileRoute, Outlet } from '@tanstack/react-router'
import { wateringPlanIdQuery } from '@/api/queries'
import EntityNotFound from '@/components/layout/EntityNotFound'
import { format } from 'date-fns'

export const Route = createFileRoute('/_protected/watering-plans/$wateringPlanId')({
  component: () => <Outlet />,
  loader: async ({ context: { queryClient }, params: { wateringPlanId } }) => {
    const wateringPlan = await queryClient.fetchQuery(wateringPlanIdQuery(wateringPlanId))
    const title = wateringPlan?.date
      ? `Einsatz: ${format(new Date(wateringPlan?.date), 'dd.MM.yyyy')}`
      : `Einsatz: ${wateringPlan.id}`
    return {
      wateringPlan,
      crumb: { title },
    }
  },
  errorComponent: () => (
    <EntityNotFound
      entityName="Einsatzplan"
      backTo="/watering-plans"
      backLabel="Zur Einsatzliste"
    />
  ),
})
