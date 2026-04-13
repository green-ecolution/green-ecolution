import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/sensors/new')({
  component: Outlet,
  loader: () => ({
    crumb: {
      title: 'Sensor hinzufügen',
    },
  }),
})
