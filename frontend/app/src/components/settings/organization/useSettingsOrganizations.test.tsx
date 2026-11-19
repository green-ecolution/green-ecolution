import { beforeEach, describe, expect, it, vi } from 'vitest'
import { renderHook } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import type {
  OrganizationResponse,
  ReachableOrganizationResponse,
  RoleResponse,
} from '@green-ecolution/backend-client'
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
  { id: 'amt', name: 'Grünflächenamt', parentId: 'root', memberCount: 3 },
  { id: 'nord', name: 'Stadtgärtnerei Nord', parentId: 'amt', memberCount: 4 },
]
const REACHABLE: ReachableOrganizationResponse[] = [
  { id: 'amt', name: 'Grünflächenamt', parentId: 'root' },
  { id: 'nord', name: 'Stadtgärtnerei Nord', parentId: 'amt' },
  { id: 'sued', name: 'Stadtgärtnerei Süd', parentId: 'amt' },
]

const role = (permissions: string[]): RoleResponse =>
  ({ id: 'r', name: 'Rolle', organizationId: 'amt', permissions }) as RoleResponse

const renderList = () => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { staleTime: Infinity, retry: false } },
  })
  queryClient.setQueryData(organizationQueries.list().queryKey, VISIBLE)
  queryClient.setQueryData(userQueries.myOrganizations().queryKey, REACHABLE)
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
    expect(renderList().organizations).toBeUndefined()
  })

  it('lists the visible organizations with organization:read', () => {
    expect(renderList()).toEqual({ organizations: VISIBLE, hasMemberCounts: true })
  })

  it('falls back to the reachable organizations without organization:read', () => {
    roles.value = [role(['start_point:read'])]
    expect(renderList()).toEqual({
      organizations: REACHABLE.map((org) => ({ ...org, memberCount: 0 })),
      hasMemberCounts: false,
    })
  })

  it('keeps the fallback inside the scope', () => {
    roles.value = [role(['start_point:read'])]
    setOrganizationScope('u1', 'nord')
    expect(renderList().organizations?.map((org) => org.id)).toEqual(['nord'])
  })
})
