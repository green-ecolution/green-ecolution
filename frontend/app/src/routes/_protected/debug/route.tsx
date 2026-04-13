import { createFileRoute, Outlet, redirect } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected/debug')({
  component: Outlet,
  beforeLoad: () => {
    // Debug routes are only available outside of production builds
    if (import.meta.env.PROD) {
      throw redirect({ to: '/', replace: true })
    }
  },
  loader: () => {
    return {
      crumb: {
        title: 'Debugging',
      },
    }
  },
})
