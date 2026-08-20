import { describe, it, expect, vi, beforeEach } from 'vitest'
import { act, waitFor } from '@testing-library/react'
import { useForm } from 'react-hook-form'
import { renderHookWithClient } from '@/test/utils'
import type { TreeclusterForm } from '@/schema/treeclusterSchema'

const getMe = vi.fn()
const listOrganizations = vi.fn()

vi.mock('@/api/backendApi', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/api/backendApi')>()),
  userApi: { getMe: () => getMe() as unknown },
  organizationApi: { listOrganizations: () => listOrganizations() as unknown },
}))

const canReadOrganizations = vi.fn(() => true)
vi.mock('@/lib/auth/useHasPermission', () => ({
  useHasPermission: () => canReadOrganizations(),
}))

vi.mock('@/lib/auth/runtimeConfig', () => ({ readAuthBypass: () => false }))

const { useClusterOrganizationSelection } = await import('./useClusterOrganizationSelection')

const orgs = [
  { id: 'root', name: 'Green Ecolution', parentId: null, memberCount: 0 },
  { id: 'tbz', name: 'Betriebshof', parentId: 'root', memberCount: 0 },
  { id: 'extern', name: 'Extern A', parentId: 'tbz', memberCount: 0 },
]

const me = (organizationId: string | null, roleOrg: string | null, permissions: string[]) => ({
  id: 'u1',
  organization: organizationId ? orgs.find((o) => o.id === organizationId) : null,
  roles: [{ id: 'r1', name: 'Rolle', organizationId: roleOrg, permissions }],
})

const renderSelection = (initialTreeIds: string[] = []) =>
  renderHookWithClient(() => {
    const form = useForm<TreeclusterForm>({ defaultValues: { treeIds: initialTreeIds } })
    return { form, selection: useClusterOrganizationSelection(form) }
  })

describe('useClusterOrganizationSelection', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    canReadOrganizations.mockReturnValue(true)
    listOrganizations.mockResolvedValue(orgs)
    getMe.mockResolvedValue(me('root', 'root', ['tree_cluster:create']))
  })

  it('preselects the users own organization', async () => {
    const { result } = renderSelection()

    await waitFor(() => expect(result.current.selection.organizationId).toBe('root'))
  })

  it('offers only organizations the user may create clusters in', async () => {
    getMe.mockResolvedValue(me('root', 'tbz', ['tree_cluster:create']))
    const { result } = renderSelection()

    await waitFor(() => expect(result.current.selection.organizations).toHaveLength(2))
    expect(result.current.selection.organizations.map((o) => o.id)).toEqual(['tbz', 'extern'])
  })

  it('falls back to the only candidate when the own organization grants nothing', async () => {
    getMe.mockResolvedValue(me('root', 'extern', ['tree_cluster:create']))
    const { result } = renderSelection()

    await waitFor(() => expect(result.current.selection.organizationId).toBe('extern'))
  })

  it('stays empty and preselects nothing without organization:read', async () => {
    canReadOrganizations.mockReturnValue(false)
    const { result } = renderSelection()

    await waitFor(() => expect(getMe).toHaveBeenCalled())
    expect(result.current.selection.organizations).toEqual([])
    expect(result.current.selection.organizationId).toBeUndefined()
  })

  it('discards already selected trees when the organization changes', async () => {
    const { result } = renderSelection(['t1', 't2'])
    await waitFor(() => expect(result.current.selection.organizationId).toBe('root'))

    act(() => result.current.selection.changeOrganization('tbz'))

    await waitFor(() => expect(result.current.selection.organizationId).toBe('tbz'))
    expect(result.current.form.getValues('treeIds')).toEqual([])
    expect(result.current.selection.discardedTreeCount).toBe(2)
  })

  it('names any visible organization, also one it cannot create in', async () => {
    getMe.mockResolvedValue(me('root', 'extern', ['tree_cluster:create']))
    const { result } = renderSelection()

    await waitFor(() => expect(result.current.selection.organizationId).toBe('extern'))
    expect(result.current.selection.nameOf('tbz')).toBe('Betriebshof')
    expect(result.current.selection.canCreateIn('tbz')).toBe(false)
    expect(result.current.selection.canCreateIn('extern')).toBe(true)
  })

  it('keeps the tree selection when the same organization is picked again', async () => {
    const { result } = renderSelection(['t1'])
    await waitFor(() => expect(result.current.selection.organizationId).toBe('root'))

    act(() => result.current.selection.changeOrganization('root'))

    expect(result.current.form.getValues('treeIds')).toEqual(['t1'])
    expect(result.current.selection.discardedTreeCount).toBe(0)
  })
})
