import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute(
  '/_protected/watering-plans/_formular/$wateringPlanId/status/edit',
)({
  component: Outlet,
  loader: () => {
    return {
      crumb: {
        title: 'Status des Einsatzplans ändern',
      },
    }
  },
})
