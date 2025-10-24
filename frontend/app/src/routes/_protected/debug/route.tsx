import { createFileRoute, Outlet } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/debug')({
  component: Outlet,
  loader: () => {
    return {
      crumb: {
        title: 'Debugging',
      },
    }
  },
})
