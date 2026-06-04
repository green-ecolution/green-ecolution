import { getUserManager, isAuthBypass } from './userManager'
import { DEMO_ACCESS_TOKEN } from './demoUser'
import { sanitizeReturnTo } from './redirect'

export interface SigninOptions {
  returnTo?: string
}

export interface AuthSession {
  getAccessToken(): Promise<string | null>
  isAuthenticated(): Promise<boolean>
  signinRedirect(opts?: SigninOptions): Promise<void>
  signinCallback(): Promise<string>
  signoutRedirect(): Promise<void>
}

export class OidcAuthSession implements AuthSession {
  private mgr = getUserManager()

  async getAccessToken(): Promise<string | null> {
    const user = await this.mgr.getUser()
    if (!user || user.expired) return null
    return user.access_token
  }

  async isAuthenticated(): Promise<boolean> {
    const user = await this.mgr.getUser()
    return !!user && !user.expired
  }

  async signinRedirect(opts?: SigninOptions): Promise<void> {
    await this.mgr.signinRedirect({ state: { returnTo: opts?.returnTo } })
  }

  async signinCallback(): Promise<string> {
    const user = await this.mgr.signinCallback()
    // state is the opaque payload we passed to signinRedirect; its shape is our own convention
    const returnTo = (user?.state as { returnTo?: string } | undefined)?.returnTo
    return sanitizeReturnTo(returnTo)
  }

  async signoutRedirect(): Promise<void> {
    await this.mgr.signoutRedirect()
  }
}

export class DemoAuthSession implements AuthSession {
  async getAccessToken(): Promise<string | null> {
    return DEMO_ACCESS_TOKEN
  }
  async isAuthenticated(): Promise<boolean> {
    return true
  }
  async signinRedirect(_opts?: SigninOptions): Promise<void> {}
  async signinCallback(): Promise<string> {
    return '/dashboard'
  }
  async signoutRedirect(): Promise<void> {}
}

let session: AuthSession | null = null

export function getAuthSession(): AuthSession {
  if (!session) session = isAuthBypass() ? new DemoAuthSession() : new OidcAuthSession()
  return session
}
