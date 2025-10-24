import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/vehicles/_formular/$vehicleId/edit')({
  component: Outlet,
  loader: () => {
    return {
      crumb: {
        title: 'Fahrzeug editieren',
      },
    }
  },
})
