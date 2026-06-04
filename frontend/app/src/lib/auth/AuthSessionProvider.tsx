import { createContext, useContext, type ReactNode } from 'react'
import { AuthProvider, useAuth } from 'react-oidc-context'
import { getUserManager, isAuthBypass } from './userManager'
import { DEMO_ACCESS_TOKEN } from './demoUser'
import { getAuthSession, type AuthSession } from './session'

interface AuthSessionContextValue {
  isAuthenticated: boolean
  accessToken: string | null
  refreshToken: string | null
  expiresAt: Date | null
  session: AuthSession
}

const AuthSessionContext = createContext<AuthSessionContextValue | null>(null)

function RealBridge({ children }: { children: ReactNode }) {
  const auth = useAuth()
  const value: AuthSessionContextValue = {
    isAuthenticated: auth.isAuthenticated,
    accessToken: auth.user?.access_token ?? null,
    refreshToken: auth.user?.refresh_token ?? null,
    expiresAt: auth.user?.expires_at ? new Date(auth.user.expires_at * 1000) : null,
    session: getAuthSession(),
  }
  return <AuthSessionContext.Provider value={value}>{children}</AuthSessionContext.Provider>
}

export function AuthSessionProvider({ children }: { children: ReactNode }) {
  if (isAuthBypass()) {
    const demo: AuthSessionContextValue = {
      isAuthenticated: true,
      accessToken: DEMO_ACCESS_TOKEN,
      refreshToken: null,
      expiresAt: null,
      session: getAuthSession(),
    }
    return <AuthSessionContext.Provider value={demo}>{children}</AuthSessionContext.Provider>
  }
  return (
    <AuthProvider
      userManager={getUserManager()}
      onSigninCallback={() => window.history.replaceState({}, '', window.location.pathname)}
    >
      <RealBridge>{children}</RealBridge>
    </AuthProvider>
  )
}

export function useAuthSession(): AuthSessionContextValue {
  const ctx = useContext(AuthSessionContext)
  if (!ctx) throw new Error('useAuthSession must be used within AuthSessionProvider')
  return ctx
}
