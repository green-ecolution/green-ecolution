import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/watering-plans/_formular/$wateringPlanId/edit')({
  component: Outlet,
  loader: () => {
    return {
      crumb: {
        title: 'Einsatzplan editieren',
      },
    }
  },
})
