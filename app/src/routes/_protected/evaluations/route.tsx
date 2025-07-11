import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/evaluations')({
  component: Outlet,
  loader: () => {
    return {
      crumb: {
        title: 'Auswertungen',
      },
    }
  },
})
