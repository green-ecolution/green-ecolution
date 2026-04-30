import { basePath } from '@/api/backendApi'
import { deriveCodeChallenge, generateCodeVerifier, storeVerifier } from '@/lib/pkce'
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
    const verifier = generateCodeVerifier()
    const challenge = await deriveCodeChallenge(verifier)
    storeVerifier(verifier)

    const redirectUrl = `${window.location.origin}/auth/callback?redirect=${encodeURIComponent(redirect)}`
    const params = new URLSearchParams({
      redirect_url: redirectUrl,
      code_challenge: challenge,
    })
    const response = await fetch(`${basePath}/api/v1/users/login?${params.toString()}`)
    if (!response.ok) {
      throw new Error(`login init failed: ${response.status}`)
    }
    const { login_url: loginUrl } = (await response.json()) as { login_url: string }
    window.location.href = loginUrl
  },
})
