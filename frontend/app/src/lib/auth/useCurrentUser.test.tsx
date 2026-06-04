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
