import { describe, expect, it, vi } from 'vitest'
import { isRedirect } from '@tanstack/react-router'

const signinRedirect = vi.fn(() => Promise.resolve())
const signoutRedirect = vi.fn(() => Promise.resolve())

vi.mock('@/lib/auth/session', () => ({
  getAuthSession: () => ({ signinRedirect, signoutRedirect }),
}))

vi.mock('@/api/queries', () => ({
  userQueries: { me: () => ({ queryKey: ['me'], queryFn: () => Promise.resolve({}) }) },
}))

const { Route: ProtectedRoute } = await import('./_protected')
const { Route: LoginRoute } = await import('./login')
const { Route: LogoutRoute } = await import('./logout')

const anonymousContext = {
  auth: { isAuthenticated: () => Promise.resolve(false), signinRedirect },
  queryClient: { ensureQueryData: vi.fn(() => Promise.resolve({})) },
}

const location = { pathname: '/map', searchStr: '' }

type AsyncHook = (opts: Record<string, unknown>) => Promise<unknown>
interface HookRoute {
  options: { beforeLoad?: unknown; loader?: unknown }
}

const beforeLoad = (route: HookRoute, opts: Record<string, unknown>) =>
  (route.options.beforeLoad as AsyncHook)(opts)
const loader = (route: HookRoute, opts: Record<string, unknown>) =>
  (route.options.loader as AsyncHook)(opts)

describe('auth redirect guards ignore route preloading', () => {
  it('does not hand an anonymous visitor to Keycloak when _protected is preloaded', async () => {
    signinRedirect.mockClear()

    const thrown = await beforeLoad(ProtectedRoute, {
      context: anonymousContext,
      location,
      preload: true,
    }).catch((error: unknown) => error)

    expect(signinRedirect).not.toHaveBeenCalled()
    expect(isRedirect(thrown)).toBe(true)
  })

  it('hands an anonymous visitor to Keycloak on a real navigation into _protected', async () => {
    signinRedirect.mockClear()

    await beforeLoad(ProtectedRoute, {
      context: anonymousContext,
      location,
      preload: false,
    })

    expect(signinRedirect).toHaveBeenCalledWith({ returnTo: '/map' })
  })

  it('does not start the login flow when /login is preloaded', async () => {
    signinRedirect.mockClear()

    await loader(LoginRoute, { deps: { redirect: undefined }, preload: true })

    expect(signinRedirect).not.toHaveBeenCalled()
  })

  it('starts the login flow on a real navigation to /login', async () => {
    signinRedirect.mockClear()

    await loader(LoginRoute, { deps: { redirect: '/map' }, preload: false })

    expect(signinRedirect).toHaveBeenCalledWith({ returnTo: '/map' })
  })

  it('does not sign the user out when /logout is preloaded', async () => {
    signoutRedirect.mockClear()

    await beforeLoad(LogoutRoute, { preload: true })

    expect(signoutRedirect).not.toHaveBeenCalled()
  })

  it('signs the user out on a real navigation to /logout', async () => {
    signoutRedirect.mockClear()

    await beforeLoad(LogoutRoute, { preload: false })

    expect(signoutRedirect).toHaveBeenCalled()
  })
})
