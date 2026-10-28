import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('@/lib/auth/session', () => ({
  getAuthSession: () => ({
    getAccessToken: () => Promise.resolve('token'),
    signinRedirect: vi.fn(),
  }),
}))

import { userApi } from './backendApi'
import {
  getOrganizationScope,
  loadOrganizationScope,
  setOrganizationScope,
} from '@/lib/auth/scope/organizationScopeStore'

const ORG = '0190a1b2-0000-7000-8000-000000000001'

const jsonResponse = (status: number, body: unknown) =>
  new Response(JSON.stringify(body), {
    status,
    headers: { 'Content-Type': 'application/json' },
  })

describe('backend request scope header', () => {
  const fetchMock = vi.fn()

  beforeEach(() => {
    localStorage.clear()
    sessionStorage.clear()
    fetchMock.mockReset()
    vi.stubGlobal('fetch', fetchMock)
    loadOrganizationScope('u1')
  })
  afterEach(() => vi.unstubAllGlobals())

  const sentHeaders = () => {
    const [, init] = fetchMock.mock.calls[0] as [unknown, RequestInit]
    return new Headers(init.headers)
  }

  it('omits the header without a scope', async () => {
    fetchMock.mockResolvedValue(jsonResponse(200, []))
    await userApi.listMyOrganizations()
    expect(sentHeaders().has('X-Organization-Scope')).toBe(false)
  })

  it('sends the header when a scope is set', async () => {
    setOrganizationScope('u1', ORG)
    fetchMock.mockResolvedValue(jsonResponse(200, []))
    await userApi.listMyOrganizations()
    expect(sentHeaders().get('X-Organization-Scope')).toBe(ORG)
    expect(sentHeaders().get('Authorization')).toBe('Bearer token')
  })

  it('clears the scope on an organization_scope.invalid response', async () => {
    vi.stubGlobal('location', { ...window.location, reload: vi.fn() })
    setOrganizationScope('u1', ORG)
    fetchMock.mockResolvedValue(
      jsonResponse(403, { error: 'nope', code: 'organization_scope.invalid' }),
    )
    await expect(userApi.listMyOrganizations()).rejects.toBeDefined()
    expect(getOrganizationScope()).toBeNull()
  })
})
