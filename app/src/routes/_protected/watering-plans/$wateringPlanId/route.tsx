import { createFileRoute, Outlet } from '@tanstack/react-router'
import { wateringPlanIdQuery } from '@/api/queries'
import { format } from 'date-fns'

export const Route = createFileRoute('/_protected/watering-plans/$wateringPlanId')({
  component: () => <Outlet />,
  loader: async ({ context: { queryClient }, params }) => {
    const wateringPlan = await queryClient.fetchQuery(wateringPlanIdQuery(params.wateringPlanId))
    const title = wateringPlan?.date
      ? `Einsatz: ${format(new Date(wateringPlan?.date), 'dd.MM.yyyy')}`
      : `Einsatz: ${wateringPlan.id}`
    return {
      crumb: { title },
    }
  },
})
