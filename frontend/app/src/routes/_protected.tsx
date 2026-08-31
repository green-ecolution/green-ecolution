import { createFileRoute, redirect } from '@tanstack/react-router'
import { userQueries } from '@/api/queries'
import { startSigninHandover } from '@/lib/auth/handover'
import { readAuthBypass } from '@/lib/auth/runtimeConfig'

export const Route = createFileRoute('/_protected')({
  beforeLoad: async ({ context, location, preload }) => {
    if (!(await context.auth.isAuthenticated())) {
      // Hovering a protected link preloads this route. Handing the visitor over
      // to Keycloak here would navigate without a click, so bail out instead --
      // a preload must never have side effects.
      if (preload) {
        throw redirect({ to: '/' })
      }
      await startSigninHandover(context.auth, location.pathname + location.searchStr)
      // Still here, so the handover was aborted. Falling through would query /me
      // unauthenticated and strand the visitor on an error page.
      throw redirect({ to: '/', replace: true })
    }
    // Nav entries and child guards read permissions synchronously from the
    // cache, so the user has to be resolved before anything inside renders.
    if (!readAuthBypass()) {
      await context.queryClient.ensureQueryData(userQueries.me())
    }
  },
})
