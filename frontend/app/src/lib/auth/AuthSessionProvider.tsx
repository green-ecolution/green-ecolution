import { useEffect, type ReactNode } from 'react'
import { AuthProvider, useAuth } from 'react-oidc-context'
import { getUserManager, isAuthBypass } from './userManager'
import { DEMO_ACCESS_TOKEN } from './demoUser'
import { getAuthSession } from './session'
import { isSessionDeadError } from './renewError'
import { AuthSessionContext, type AuthSessionContextValue } from './authSessionContext'

function RealBridge({ children }: { children: ReactNode }) {
  const auth = useAuth()
  useEffect(
    () =>
      auth.events.addSilentRenewError((err) => {
        if (!isSessionDeadError(err)) return
        void auth.removeUser().then(() =>
          getAuthSession().signinRedirect({
            returnTo: window.location.pathname + window.location.search,
          }),
        )
      }),
    [auth],
  )
  const value: AuthSessionContextValue = {
    isAuthenticated: auth.isAuthenticated,
    accessToken: auth.user?.access_token ?? null,
    refreshToken: auth.user?.refresh_token ?? null,
    expiresAt: auth.user?.expires_at ? new Date(auth.user.expires_at * 1000) : null,
    session: getAuthSession(),
  }
  return <AuthSessionContext value={value}>{children}</AuthSessionContext>
}

export function AuthSessionProvider({ children }: { children: ReactNode }) {
  // VITE_AUTH_BYPASS is a build-time constant, so this branch is stable across the component's lifetime.
  if (isAuthBypass()) {
    const demo: AuthSessionContextValue = {
      isAuthenticated: true,
      accessToken: DEMO_ACCESS_TOKEN,
      refreshToken: null,
      expiresAt: null,
      session: getAuthSession(),
    }
    return <AuthSessionContext value={demo}>{children}</AuthSessionContext>
  }
  return (
    // /auth/callback owns signinCallback; without skip the provider races it on the one-shot state
    <AuthProvider userManager={getUserManager()} skipSigninCallback>
      <RealBridge>{children}</RealBridge>
    </AuthProvider>
  )
}
