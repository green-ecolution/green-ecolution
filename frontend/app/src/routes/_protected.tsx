import { createFileRoute } from '@tanstack/react-router'

export const Route = createFileRoute('/_protected')({
  beforeLoad: async ({ context, location }) => {
    if (!(await context.auth.isAuthenticated())) {
      await context.auth.signinRedirect({ returnTo: location.pathname + location.searchStr })
    }
  },
})
