import { basePath } from '@/api/backendApi'
import { takeVerifier } from '@/lib/pkce'
import useStore from '@/store/store'
import { ClientTokenResponseFromJSON } from '@green-ecolution/backend-client'
import { createFileRoute, redirect as routerRedirect } from '@tanstack/react-router'
import { z } from 'zod'

const authSearchParamsSchema = z.object({
  session_state: z.string().optional(),
  iss: z.string().optional(),
  code: z.string(),
  redirect: z.string(),
})

export const Route = createFileRoute('/auth/callback')({
  validateSearch: authSearchParamsSchema,
  loaderDeps: ({ search: { code } }) => ({ code }),
  beforeLoad: async ({ search: { code, redirect } }) => {
    const verifier = takeVerifier()
    const redirectUrl = `${window.location.origin}/auth/callback?redirect=${encodeURIComponent(redirect)}`

    const response = await fetch(
      `${basePath}/api/v1/users/login/token?redirect_url=${encodeURIComponent(redirectUrl)}`,
      {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ code, code_verifier: verifier ?? undefined }),
      },
    ).catch((err: unknown) => {
      console.error('login token request failed', err)
      throw err instanceof Error ? err : new Error('login token request failed')
    })

    if (!response.ok) {
      const body = await response.text().catch(() => '')
      console.error('login token request failed', response.status, body)
      throw new Error(`login token request failed: ${response.status}`)
    }

    const token = ClientTokenResponseFromJSON(await response.json())
    useStore.getState().setToken(token)
    useStore.getState().setUserFromJwt(token.accessToken)

    throw routerRedirect({ to: redirect, replace: true })
  },
})
