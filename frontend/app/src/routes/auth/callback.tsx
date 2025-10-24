import { userApi } from '@/api/backendApi'
import useStore from '@/store/store'
import { createFileRoute, redirect as routerRedirect } from '@tanstack/react-router'
import { z } from 'zod'

const authSearchParamsSchema = z.object({
  session_state: z.string(),
  iss: z.string(),
  code: z.string(),
  redirect: z.string(),
})

export const Route = createFileRoute('/auth/callback')({
  validateSearch: authSearchParamsSchema,
  loaderDeps: ({ search: { code } }) => ({ code }),
  beforeLoad: async ({ search: { code, redirect } }) => {
    const token = await userApi
      .v1UserLoginTokenPost({
        redirectUrl: `${window.location.origin}/auth/callback?redirect=${encodeURIComponent(redirect)}`,
        body: {
          code,
        },
      })
      .catch((err) => {
        if (err instanceof Error) {
          console.error(err.message)
          throw new Error(err.message)
        } else {
          console.error('An unknown error occurred', err)
          throw new Error('An unknown error occurred')
        }
      })

    if (!token) {
      console.error('Error while fetching token')
      throw new Error('Error while fetching token')
    }

    useStore.getState().auth.setToken(token)
    useStore.getState().user.setFromJwt(token.accessToken)

    throw routerRedirect({
      to: redirect,
      replace: true,
    })
  },
})
