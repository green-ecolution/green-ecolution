import { createContext, use } from 'react'
import type { AuthSession } from './session'

export interface AuthSessionContextValue {
  isAuthenticated: boolean
  accessToken: string | null
  refreshToken: string | null
  expiresAt: Date | null
  session: AuthSession
}

export const AuthSessionContext = createContext<AuthSessionContextValue | null>(null)

export function useAuthSession(): AuthSessionContextValue {
  const ctx = use(AuthSessionContext)
  if (!ctx) throw new Error('useAuthSession must be used within AuthSessionProvider')
  return ctx
}
