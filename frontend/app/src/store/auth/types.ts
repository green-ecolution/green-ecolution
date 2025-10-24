import { ClientToken } from '@green-ecolution/backend-client'

interface AuthState {
  isAuthenticated: boolean
  token: ClientToken | null
}

interface AuthActions {
  setIsAuthenticated: (auth: boolean) => void
  setToken: (token: ClientToken) => void
  clear: () => void
}

export type AuthStore = AuthState & AuthActions
