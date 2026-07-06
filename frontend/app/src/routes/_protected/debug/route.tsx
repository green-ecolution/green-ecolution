import { createFileRoute, redirect } from '@tanstack/react-router'
import { crumbRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/debug')({
  ...crumbRoute('Debugging'),
  beforeLoad: () => {
    // Debug routes are only available outside of production builds
    if (import.meta.env.PROD) {
      throw redirect({ to: '/', replace: true })
    }
  },
})
