import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/vehicles')({
  component: Outlet,
  loader: () => {
    return {
      crumb: {
        title: 'Fahrzeuge',
      },
    }
  },
})
