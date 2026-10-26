import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook } from '@testing-library/react'
import { UNKNOWN_USER_STATUS } from '@/hooks/details/useDetailsForUserStatus'
import { AuthSessionProvider } from './AuthSessionProvider'
import { AuthSessionContext } from './authSessionContext'
import { DemoAuthSession } from './session'
import { useCurrentUser } from './useCurrentUser'

beforeEach(() => {
  vi.stubEnv('VITE_AUTH_BYPASS', 'true')
})

describe('useCurrentUser (bypass)', () => {
  it('returns the demo profile fields', () => {
    const { result } = renderHook(() => useCurrentUser(), {
      wrapper: ({ children }) => <AuthSessionProvider>{children}</AuthSessionProvider>,
    })
    expect(result.current.username).toBe('ttester')
    expect(result.current.email).toBe('toni.tester@green-ecolution.de')
    expect(result.current.firstName).toBe('Toni')
  })
})

describe('useCurrentUser (token without profile attributes)', () => {
  it('falls back to unknown status when the JWT carries no status claim', () => {
    const payload = {
      preferred_username: 'tbz1',
      email: 'tbz1@example.com',
      given_name: 'T',
      family_name: 'BZ',
    }
    const token = `x.${btoa(JSON.stringify(payload))}.y`
    const { result } = renderHook(() => useCurrentUser(), {
      wrapper: ({ children }) => (
        <AuthSessionContext
          value={{
            isAuthenticated: true,
            accessToken: token,
            refreshToken: null,
            expiresAt: null,
            session: new DemoAuthSession(),
          }}
        >
          {children}
        </AuthSessionContext>
      ),
    })
    expect(result.current.username).toBe('tbz1')
    expect(result.current.userStatus).toBe(UNKNOWN_USER_STATUS)
    expect(result.current.drivingLicenses).toEqual([])
  })
})
