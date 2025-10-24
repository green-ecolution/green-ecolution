import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/team')({
  component: Outlet,
  loader: () => {
    return {
      crumb: {
        title: 'Mitarbeitende',
      },
    }
  },
})
