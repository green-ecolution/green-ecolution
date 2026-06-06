import { getAuthSession } from '@/lib/auth/session'
import { createFileRoute, redirect as routerRedirect } from '@tanstack/react-router'

export const Route = createFileRoute('/auth/callback')({
  beforeLoad: async () => {
    const returnTo = await getAuthSession().signinCallback()
    throw routerRedirect({ to: returnTo, replace: true })
  },
})
