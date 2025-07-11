import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/watering-plans/_formular/new')({
  component: Outlet,
  loader: () => {
    return {
      crumb: {
        title: 'Neuen Einsatzplan',
      },
    }
  },
})
