import { UserManager, WebStorageStateStore, type UserManagerSettings } from 'oidc-client-ts'

export function isAuthBypass(): boolean {
  return import.meta.env.VITE_AUTH_BYPASS === 'true'
}

function buildSettings(): UserManagerSettings {
  const authority = import.meta.env.VITE_OIDC_AUTHORITY
  const client_id = import.meta.env.VITE_OIDC_CLIENT_ID
  if (!authority || !client_id) {
    throw new Error('VITE_OIDC_AUTHORITY and VITE_OIDC_CLIENT_ID are required when auth is enabled')
  }
  const origin = window.location.origin
  return {
    authority,
    client_id,
    redirect_uri: `${origin}/auth/callback`,
    post_logout_redirect_uri: `${origin}/`,
    response_type: 'code',
    scope: 'openid profile email',
    userStore: new WebStorageStateStore({ store: window.localStorage }),
    automaticSilentRenew: true,
    // Cross-domain iframe silent renew is blocked by third-party cookie rules;
    // renew runs off the rotating refresh token instead.
    monitorSession: false,
  }
}

let manager: UserManager | null = null

export function getUserManager(): UserManager {
  if (!manager) manager = new UserManager(buildSettings())
  return manager
}
