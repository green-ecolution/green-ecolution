import { createFileRoute } from '@tanstack/react-router'
import { currentUserQuery } from '@/api/queries'
import { readAuthBypass } from '@/lib/auth/runtimeConfig'

export const Route = createFileRoute('/_protected')({
  beforeLoad: async ({ context, location }) => {
    if (!(await context.auth.isAuthenticated())) {
      await context.auth.signinRedirect({ returnTo: location.pathname + location.searchStr })
    }
    // Nav entries and child guards read permissions synchronously from the
    // cache, so the user has to be resolved before anything inside renders.
    if (!readAuthBypass()) {
      await context.queryClient.ensureQueryData(currentUserQuery())
    }
  },
})
