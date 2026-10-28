import { describe, it, expect, vi, beforeEach } from 'vitest'
import { waitFor } from '@testing-library/react'
import { renderHookWithClient } from '@/test/utils'

const getMe = vi.fn()
const listOrganizations = vi.fn()

vi.mock('@/api/backendApi', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/api/backendApi')>()),
  userApi: { getMe: () => getMe() as unknown, listMyOrganizations: () => [] },
  organizationApi: { listOrganizations: () => listOrganizations() as unknown },
}))

const canReadOrganizations = vi.fn(() => true)
vi.mock('@/lib/auth/useHasPermission', () => ({
  useHasPermission: () => canReadOrganizations(),
}))

const bypass = vi.fn(() => false)
vi.mock('@/lib/auth/runtimeConfig', () => ({ readAuthBypass: () => bypass() }))

vi.mock('@/lib/auth/authSessionContext', () => ({
  useAuthSession: () => ({ isAuthenticated: true }),
}))

const { useTransferTargets } = await import('./useTransferTargets')
const { setOrganizationScope } = await import('@/lib/auth/scope/organizationScopeStore')

const orgs = [
  { id: 'root', name: 'Green Ecolution', parentId: null, memberCount: 0 },
  { id: 'tbz', name: 'Betriebshof', parentId: 'root', memberCount: 0 },
  { id: 'extern', name: 'Extern A', parentId: 'tbz', memberCount: 0 },
  { id: 'other', name: 'Andere Stadt', parentId: 'root', memberCount: 0 },
]

const me = (roleOrg: string | null, permissions: string[]) => ({
  id: 'u1',
  organization: orgs[0],
  roles: [{ id: 'r1', name: 'Rolle', organizationId: roleOrg, permissions }],
})

describe('useTransferTargets', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    setOrganizationScope('u1', null)
    canReadOrganizations.mockReturnValue(true)
    bypass.mockReturnValue(false)
    listOrganizations.mockResolvedValue(orgs)
    getMe.mockResolvedValue(me('tbz', ['tree_cluster:update']))
  })

  it('offers the organizations holding the update right, minus the current one', async () => {
    const { result } = renderHookWithClient(() => useTransferTargets('tree_cluster:update', 'tbz'))

    await waitFor(() => expect(result.current.map((org) => org.id)).toEqual(['extern']))
  })

  it('offers nothing when the right is held elsewhere', async () => {
    getMe.mockResolvedValue(me('tbz', ['tree:update']))
    const { result } = renderHookWithClient(() => useTransferTargets('tree_cluster:update', 'tbz'))

    await waitFor(() => expect(listOrganizations).toHaveBeenCalled())
    expect(result.current).toEqual([])
  })

  it('stays empty without organization:read', () => {
    canReadOrganizations.mockReturnValue(false)
    const { result } = renderHookWithClient(() => useTransferTargets('tree_cluster:update', 'tbz'))

    expect(result.current).toEqual([])
    expect(listOrganizations).not.toHaveBeenCalled()
  })

  it('offers every other organization under the auth bypass', async () => {
    bypass.mockReturnValue(true)
    getMe.mockResolvedValue(me(null, []))
    const { result } = renderHookWithClient(() => useTransferTargets('tree_cluster:update', 'root'))

    await waitFor(() =>
      expect(result.current.map((org) => org.name)).toEqual([
        'Andere Stadt',
        'Betriebshof',
        'Extern A',
      ]),
    )
  })
})
