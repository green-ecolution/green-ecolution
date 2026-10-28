import { createFileRoute, redirect } from '@tanstack/react-router'
import { isAdminMode } from '@/lib/auth/scope/adminMode'
import { crumbRoute } from '@/lib/router'

export const Route = createFileRoute('/_protected/admin')({
  ...crumbRoute('admin'),
  // Reachable by URL for everyone, meaningful only in admin mode.
  beforeLoad: async ({ context }) => {
    if (!(await isAdminMode(context.queryClient))) {
      throw redirect({ to: '/dashboard', replace: true })
    }
  },
})
