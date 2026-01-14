import { describe, it, expect, beforeEach, vi } from 'vitest'
import { UserStatus, DrivingLicense, UserRole } from '@green-ecolution/backend-client'
import useStore from './store'

// Mock localStorage
const localStorageMock = (() => {
  let store: Record<string, string> = {}
  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key]
    }),
    clear: () => {
      store = {}
    },
  }
})()

Object.defineProperty(window, 'localStorage', { value: localStorageMock })

// Helper to create a mock JWT
const createMockJwt = (payload: object): string => {
  const header = btoa(JSON.stringify({ alg: 'RS256', typ: 'JWT' }))
  const body = btoa(JSON.stringify(payload))
  const signature = 'mock-signature'
  return `${header}.${body}.${signature}`
}

describe('Store - Auth Slice', () => {
  beforeEach(() => {
    localStorageMock.clear()
    vi.clearAllMocks()
    useStore.setState({
      isAuthenticated: false,
      token: null,
    })
  })

  it('setToken sets token and isAuthenticated to true', () => {
    const mockToken = {
      accessToken: 'access-token',
      refreshToken: 'refresh-token',
      expiresIn: 3600,
      tokenType: 'Bearer',
      expiry: '2025-01-01T00:00:00Z',
      idToken: 'id-token',
      notBeforePolicy: 0,
      refreshExpiresIn: 7200,
      scope: 'openid',
      sessionState: 'session-state',
    }

    useStore.getState().setToken(mockToken)

    expect(useStore.getState().isAuthenticated).toBe(true)
    expect(useStore.getState().token).toEqual(mockToken)
    expect(localStorageMock.setItem).toHaveBeenCalledWith('refreshToken', 'refresh-token')
  })

  it('clearAuth clears token and sets isAuthenticated to false', () => {
    useStore.setState({
      isAuthenticated: true,
      token: {
        accessToken: 'access-token',
        refreshToken: 'refresh-token',
        expiresIn: 3600,
        tokenType: 'Bearer',
        expiry: '2025-01-01T00:00:00Z',
        idToken: 'id-token',
        notBeforePolicy: 0,
        refreshExpiresIn: 7200,
        scope: 'openid',
        sessionState: 'session-state',
      },
    })

    useStore.getState().clearAuth()

    expect(useStore.getState().isAuthenticated).toBe(false)
    expect(useStore.getState().token).toBeNull()
    expect(localStorageMock.removeItem).toHaveBeenCalledWith('refreshToken')
  })
})

describe('Store - User Slice', () => {
  beforeEach(() => {
    useStore.setState({
      username: '',
      email: '',
      firstName: '',
      lastName: '',
      drivingLicenses: [],
      userRoles: [],
      userStatus: UserStatus.UserStatusUnknown,
    })
  })

  it('setUserFromJwt parses JWT and sets user data', () => {
    const mockJwt = createMockJwt({
      preferred_username: 'testuser',
      email: 'test@example.com',
      given_name: 'Test',
      family_name: 'User',
      driving_licenses: ['BE'],
      user_roles: ['tbz'],
      status: 'available',
    })

    useStore.getState().setUserFromJwt(mockJwt)

    expect(useStore.getState().username).toBe('testuser')
    expect(useStore.getState().email).toBe('test@example.com')
    expect(useStore.getState().firstName).toBe('Test')
    expect(useStore.getState().lastName).toBe('User')
    expect(useStore.getState().drivingLicenses).toEqual([DrivingLicense.DrivingLicenseBE])
    expect(useStore.getState().userRoles).toEqual([UserRole.UserRoleTbz])
    expect(useStore.getState().userStatus).toBe(UserStatus.UserStatusAvailable)
  })

  it('setUserFromJwt handles missing optional fields', () => {
    const mockJwt = createMockJwt({
      preferred_username: 'testuser',
      email: 'test@example.com',
      given_name: 'Test',
      family_name: 'User',
      status: 'unknown',
    })

    useStore.getState().setUserFromJwt(mockJwt)

    expect(useStore.getState().username).toBe('testuser')
    expect(useStore.getState().drivingLicenses).toEqual([])
    expect(useStore.getState().userRoles).toEqual([])
    expect(useStore.getState().userStatus).toBe(UserStatus.UserStatusUnknown)
  })

  it('isUserEmpty returns true when user data is empty', () => {
    expect(useStore.getState().isUserEmpty()).toBe(true)
  })

  it('isUserEmpty returns true when only some fields are set', () => {
    useStore.setState({
      username: 'testuser',
      email: 'test@example.com',
      firstName: '',
      lastName: '',
    })

    expect(useStore.getState().isUserEmpty()).toBe(true)
  })

  it('isUserEmpty returns false when all required fields are set', () => {
    useStore.setState({
      username: 'testuser',
      email: 'test@example.com',
      firstName: 'Test',
      lastName: 'User',
      drivingLicenses: [DrivingLicense.DrivingLicenseB],
      userRoles: [UserRole.UserRoleTbz],
      userStatus: UserStatus.UserStatusAvailable,
    })

    expect(useStore.getState().isUserEmpty()).toBe(false)
  })

  it('clearUser resets all user fields', () => {
    useStore.setState({
      username: 'testuser',
      email: 'test@example.com',
      firstName: 'Test',
      lastName: 'User',
      drivingLicenses: [DrivingLicense.DrivingLicenseB],
      userRoles: [UserRole.UserRoleTbz],
      userStatus: UserStatus.UserStatusAvailable,
    })

    useStore.getState().clearUser()

    expect(useStore.getState().username).toBe('')
    expect(useStore.getState().email).toBe('')
    expect(useStore.getState().firstName).toBe('')
    expect(useStore.getState().lastName).toBe('')
    expect(useStore.getState().drivingLicenses).toEqual([])
    expect(useStore.getState().userRoles).toEqual([])
    expect(useStore.getState().userStatus).toBe(UserStatus.UserStatusUnknown)
  })
})
