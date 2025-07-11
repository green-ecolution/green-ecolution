import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/profile')({
  component: Outlet,
  loader: () => {
    return {
      crumb: {
        title: 'Profil',
      },
    }
  },
})
