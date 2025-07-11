import { userApi } from '@/api/backendApi'
import { createFileRoute } from '@tanstack/react-router'
import { z } from 'zod'

const loginSchema = z.object({
  redirect: z.string().optional(),
})

export const Route = createFileRoute('/login')({
  validateSearch: loginSchema,
  loaderDeps: ({ search: { redirect } }) => ({
    redirect: redirect ?? '/dashboard',
  }),
  loader: async ({ deps: { redirect } }) => {
    const loginUrl = await userApi
      .v1UserLoginGet({
        redirectUrl: `${window.location.origin}/auth/callback?redirect=${encodeURIComponent(redirect)}`,
      })
      .then((res) => res.loginUrl)

    window.location.href = loginUrl
  },
})
