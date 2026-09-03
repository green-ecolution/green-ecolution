interface RuntimeEnv {
  VITE_AUTH_BYPASS?: string
  VITE_OIDC_AUTHORITY?: string
  VITE_OIDC_CLIENT_ID?: string
  VITE_ANALYTICS_SCRIPT_URL?: string
}

function runtimeEnv(): RuntimeEnv {
  return (window as unknown as { _env_?: RuntimeEnv })._env_ ?? {}
}

export function readAuthBypass(): boolean {
  return (runtimeEnv().VITE_AUTH_BYPASS ?? import.meta.env.VITE_AUTH_BYPASS) === 'true'
}

export function readOidcAuthority(): string | undefined {
  return runtimeEnv().VITE_OIDC_AUTHORITY ?? import.meta.env.VITE_OIDC_AUTHORITY
}

export function readOidcClientId(): string | undefined {
  return runtimeEnv().VITE_OIDC_CLIENT_ID ?? import.meta.env.VITE_OIDC_CLIENT_ID
}

/**
 * Runtime-only on purpose, with no build-time fallback: which instance is
 * measured is a deployment decision, and a baked-in value would follow the
 * shared image into every other environment.
 */
export function readAnalyticsScriptUrl(): string | undefined {
  return runtimeEnv().VITE_ANALYTICS_SCRIPT_URL
}
