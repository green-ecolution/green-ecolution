import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/vehicles/_formular/new')({
  component: Outlet,
  loader: () => {
    return {
      crumb: {
        title: 'Neues Fahrzeug',
      },
    }
  },
})
