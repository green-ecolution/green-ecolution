import { getAuthSession } from '@/lib/auth/session'
import { sanitizeReturnTo } from '@/lib/auth/redirect'
import { createFileRoute } from '@tanstack/react-router'
import { z } from 'zod'

const loginSchema = z.object({
  redirect: z.string().optional(),
})

export const Route = createFileRoute('/login')({
  validateSearch: loginSchema,
  loaderDeps: ({ search: { redirect } }) => ({ redirect }),
  loader: async ({ deps: { redirect }, preload }) => {
    // A hover-triggered preload must not leave the page; only a real click may.
    if (preload) {
      return
    }
    await getAuthSession().signinRedirect({ returnTo: sanitizeReturnTo(redirect) })
  },
})
