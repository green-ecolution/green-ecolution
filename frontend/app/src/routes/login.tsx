import { getAuthSession } from '@/lib/auth/session'
import { startSigninHandover } from '@/lib/auth/handover'
import { sanitizeReturnTo } from '@/lib/auth/redirect'
import { createFileRoute, redirect } from '@tanstack/react-router'
import { z } from 'zod'

const loginSchema = z.object({
  redirect: z.string().optional(),
})

export const Route = createFileRoute('/login')({
  validateSearch: loginSchema,
  loaderDeps: ({ search: { redirect } }) => ({ redirect }),
  loader: async ({ deps: { redirect: returnTo }, preload }) => {
    // A hover-triggered preload must not leave the page; only a real click may.
    if (preload) {
      return
    }
    await startSigninHandover(getAuthSession(), sanitizeReturnTo(returnTo))
    // Still here, so the handover was aborted. This route renders nothing, so
    // without leaving it the visitor would be looking at a blank page.
    throw redirect({ to: '/', replace: true })
  },
})
