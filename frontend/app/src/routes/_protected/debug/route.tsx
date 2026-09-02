import { createFileRoute, Outlet, redirect } from '@tanstack/react-router'

// Developer surface, deliberately excluded from translation.
export const Route = createFileRoute('/_protected/debug')({
  component: Outlet,
  loader: () => ({ crumb: { title: 'Debugging' } }),
  beforeLoad: () => {
    // Debug routes are only available outside of production builds
    if (import.meta.env.PROD) {
      throw redirect({ to: '/', replace: true })
    }
  },
})
