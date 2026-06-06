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
  loader: async ({ deps: { redirect } }) => {
    await getAuthSession().signinRedirect({ returnTo: sanitizeReturnTo(redirect) })
  },
})
