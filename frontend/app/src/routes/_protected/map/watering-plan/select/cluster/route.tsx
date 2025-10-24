import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/map/watering-plan/select/cluster')({
  component: Outlet,
  loader: () => {
    return {
      crumb: { title: 'Route festlegen' },
    }
  },
})
