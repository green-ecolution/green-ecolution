import { beforeEach, describe, expect, it, vi } from 'vitest'
import { renderHook } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import type {
  OrganizationResponse,
  ReachableOrganizationResponse,
  RoleResponse,
} from '@green-ecolution/backend-client'
import { organizationApi } from '@/api/backendApi'
import { organizationQueries, userQueries } from '@/api/queries'
import { setOrganizationScope } from '@/lib/auth/scope/organizationScopeStore'

const roles = vi.hoisted(() => ({ value: undefined as RoleResponse[] | undefined }))
vi.mock('@/lib/auth/scope/effectiveRoles', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/lib/auth/scope/effectiveRoles')>()),
  useEffectiveRoles: () => roles.value,
}))
vi.mock('@/lib/auth/runtimeConfig', () => ({ readAuthBypass: () => false }))

const { useSettingsOrganizations } = await import('./useSettingsOrganizations')

const VISIBLE: OrganizationResponse[] = [
  { id: 'amt', name: 'Grünflächenamt', parentId: 'root', memberCount: 3, contactPersonId: 'u1' },
  { id: 'nord', name: 'Stadtgärtnerei Nord', parentId: 'amt', memberCount: 4 },
]
const REACHABLE: ReachableOrganizationResponse[] = [
  { id: 'amt', name: 'Grünflächenamt', parentId: 'root' },
  { id: 'nord', name: 'Stadtgärtnerei Nord', parentId: 'amt' },
  { id: 'sued', name: 'Stadtgärtnerei Süd', parentId: 'amt' },
]

const role = (permissions: string[]): RoleResponse =>
  ({ id: 'r', name: 'Rolle', organizationId: 'amt', permissions }) as RoleResponse

const renderList = (data: {
  visible?: OrganizationResponse[]
  reachable: ReachableOrganizationResponse[]
}) => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { staleTime: Infinity, retry: false } },
  })
  if (data.visible) queryClient.setQueryData(organizationQueries.list().queryKey, data.visible)
  queryClient.setQueryData(userQueries.myOrganizations().queryKey, data.reachable)
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  )
  return renderHook(() => useSettingsOrganizations(), { wrapper }).result.current
}

describe('useSettingsOrganizations', () => {
  beforeEach(() => {
    roles.value = [role(['organization:read'])]
    setOrganizationScope('u1', null)
  })

  it('is unknown while the roles load', () => {
    roles.value = undefined
    expect(renderList({ visible: VISIBLE, reachable: REACHABLE }).organizations).toBeUndefined()
  })

  it('lists only GET /organizations when every reachable organization is readable', () => {
    expect(renderList({ visible: VISIBLE, reachable: REACHABLE.slice(0, 2) })).toEqual({
      organizations: VISIBLE,
      readableIds: new Set(['amt', 'nord']),
    })
  })

  it('lists the reachable organizations without organization:read and never asks for the rest', () => {
    const listOrganizations = vi.spyOn(organizationApi, 'listOrganizations')
    roles.value = [role(['start_point:read'])]
    expect(renderList({ reachable: REACHABLE })).toEqual({
      organizations: REACHABLE.map((org) => ({ ...org, memberCount: 0 })),
      readableIds: new Set(),
    })
    expect(listOrganizations).not.toHaveBeenCalled()
  })

  it('adds the reachable organizations GET /organizations lacks, without a member count', () => {
    expect(renderList({ visible: VISIBLE, reachable: REACHABLE })).toEqual({
      organizations: [
        ...VISIBLE,
        { id: 'sued', name: 'Stadtgärtnerei Süd', parentId: 'amt', memberCount: 0 },
      ],
      readableIds: new Set(['amt', 'nord']),
    })
  })

  it('adds only reachable organizations inside the scope', () => {
    setOrganizationScope('u1', 'nord')
    const { organizations } = renderList({ visible: [VISIBLE[1]], reachable: REACHABLE })
    expect(organizations?.map((org) => org.id)).toEqual(['nord'])
  })
})
