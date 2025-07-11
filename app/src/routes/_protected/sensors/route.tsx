import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/sensors')({
  component: Outlet,
  loader: () => {
    return {
      crumb: {
        title: 'Sensoren',
      },
    }
  },
})
