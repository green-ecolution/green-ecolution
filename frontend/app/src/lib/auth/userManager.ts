import { UserManager, WebStorageStateStore, type UserManagerSettings } from 'oidc-client-ts'
import { readAuthBypass, readOidcAuthority, readOidcClientId } from './runtimeConfig'

export function isAuthBypass(): boolean {
  return readAuthBypass()
}

function buildSettings(): UserManagerSettings {
  const authority = readOidcAuthority()
  const client_id = readOidcClientId()
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
    // offline_access keeps suspended PWAs renewable beyond the SSO idle timeout
    scope: 'openid profile email offline_access',
    userStore: new WebStorageStateStore({ store: window.localStorage }),
    automaticSilentRenew: true,
    // Cross-domain iframe silent renew is blocked by third-party cookie rules;
    // renew runs off the rotating refresh token instead.
    monitorSession: false,
  }
}

let manager: UserManager | null = null

export function getUserManager(): UserManager {
  manager ??= new UserManager(buildSettings())
  return manager
}
