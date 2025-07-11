import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/settings/plugin')({
  component: Outlet,
  loader: () => {
    return {
      crumb: {
        title: 'Plugins',
      },
    }
  },
})
