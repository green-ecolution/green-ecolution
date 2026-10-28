import { createFileRoute, redirect } from '@tanstack/react-router'
import { modulesQuery, userQueries } from '@/api/queries'
import ScopedOutlet from '@/components/layout/ScopedOutlet'
import { startSigninHandover } from '@/lib/auth/handover'
import { DEMO_USER_ID, loadOrganizationScope } from '@/lib/auth/scope/organizationScopeStore'
import { readAuthBypass } from '@/lib/auth/runtimeConfig'
import { prefetch } from '@/lib/router'

export const Route = createFileRoute('/_protected')({
  component: ScopedOutlet,
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
    if (readAuthBypass()) {
      loadOrganizationScope(DEMO_USER_ID)
    } else {
      const me = await context.queryClient.ensureQueryData(userQueries.me())
      // Requests after this point carry the user's stored organization scope.
      loadOrganizationScope(me.id)
    }
    // Module guards and the navigation read the active modules synchronously;
    // the endpoint answers in auth-bypass mode too.
    await context.queryClient.ensureQueryData(modulesQuery())
    // Every map below here opens at this viewport and suspends on it. Warming
    // it once at the top means no page pays for it individually, and the small
    // previews never render a frame at the wrong place.
    prefetch(context.queryClient, userQueries.mapView(), 'userQueries.mapView')
  },
})
