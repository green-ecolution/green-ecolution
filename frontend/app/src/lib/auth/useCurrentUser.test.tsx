import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook } from '@testing-library/react'

beforeEach(() => {
  vi.stubEnv('VITE_AUTH_BYPASS', 'true')
  vi.resetModules()
})

describe('useCurrentUser (bypass)', () => {
  it('returns the demo profile fields', async () => {
    const { AuthSessionProvider } = await import('./AuthSessionProvider')
    const { useCurrentUser } = await import('./useCurrentUser')
    const { result } = renderHook(() => useCurrentUser(), {
      wrapper: ({ children }) => <AuthSessionProvider>{children}</AuthSessionProvider>,
    })
    expect(result.current.username).toBe('ttester')
    expect(result.current.email).toBe('toni.tester@green-ecolution.de')
    expect(result.current.firstName).toBe('Toni')
    expect(result.current.userRoles.length).toBeGreaterThan(0)
  })
})

describe('useCurrentUser (token without profile attributes)', () => {
  it('falls back to unknown status when the JWT carries no status claim', async () => {
    const payload = {
      preferred_username: 'tbz1',
      email: 'tbz1@example.com',
      given_name: 'T',
      family_name: 'BZ',
      user_roles: ['tbz'],
    }
    const token = `x.${btoa(JSON.stringify(payload))}.y`
    vi.doMock('./authSessionContext', () => ({
      // eslint-disable-next-line react-x/no-unnecessary-use-prefix -- mock must keep the real hook's exported name
      useAuthSession: () => ({ accessToken: token }),
    }))
    const { useCurrentUser } = await import('./useCurrentUser')
    const { UNKNOWN_USER_STATUS } = await import('@/hooks/details/useDetailsForUserStatus')
    const { result } = renderHook(() => useCurrentUser())
    expect(result.current.username).toBe('tbz1')
    expect(result.current.userStatus).toBe(UNKNOWN_USER_STATUS)
    expect(result.current.drivingLicenses).toEqual([])
    vi.doUnmock('./authSessionContext')
  })
})
