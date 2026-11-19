import { beforeEach, describe, expect, it, vi } from 'vitest'
import { renderHook } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import type { ReachableOrganizationResponse, UserResponse } from '@green-ecolution/backend-client'
import { userQueries } from '@/api/queries'
import { setOrganizationScope } from './organizationScopeStore'

const bypass = vi.hoisted(() => ({ value: false }))
vi.mock('@/lib/auth/runtimeConfig', () => ({ readAuthBypass: () => bypass.value }))

const { resolveSettingsOrganizationId, useSettingsHomeOrganizationId } =
  await import('./settingsOrganization')

const REACHABLE: ReachableOrganizationResponse[] = [
  { id: 'nord', name: 'Stadtgärtnerei Nord', parentId: 'amt' },
  { id: 'amt', name: 'Grünflächenamt', parentId: 'root' },
  { id: 'betrieb', name: 'Betriebshof', parentId: 'root' },
]

describe('resolveSettingsOrganizationId', () => {
  it('takes the scope while it is reachable', () => {
    expect(resolveSettingsOrganizationId('nord', 'amt', REACHABLE)).toBe('nord')
  })

  it('ignores a scope that is no longer reachable', () => {
    expect(resolveSettingsOrganizationId('deleted', 'amt', REACHABLE)).toBe('amt')
  })

  it('takes the own organization while it is reachable', () => {
    expect(resolveSettingsOrganizationId(null, 'nord', REACHABLE)).toBe('nord')
  })

  it('falls back to the topmost reachable organization', () => {
    expect(resolveSettingsOrganizationId(null, 'elsewhere', REACHABLE)).toBe('betrieb')
    expect(resolveSettingsOrganizationId(null, undefined, REACHABLE)).toBe('betrieb')
  })

  it('resolves nothing without any reachable organization', () => {
    expect(resolveSettingsOrganizationId('nord', 'amt', [])).toBeNull()
  })
})

const me = { id: 'u1', organization: { id: 'amt', name: 'Grünflächenamt' } } as UserResponse

const renderHome = (data: { me?: UserResponse; reachable?: ReachableOrganizationResponse[] }) => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { staleTime: Infinity, retry: false, enabled: false } },
  })
  if (data.me) queryClient.setQueryData(userQueries.me().queryKey, data.me)
  if (data.reachable) {
    queryClient.setQueryData(userQueries.myOrganizations().queryKey, data.reachable)
  }
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  )
  return renderHook(() => useSettingsHomeOrganizationId(), { wrapper }).result.current
}

describe('useSettingsHomeOrganizationId', () => {
  beforeEach(() => {
    bypass.value = false
    setOrganizationScope('u1', null)
  })

  it('is undefined while the data loads', () => {
    expect(renderHome({ me })).toBeUndefined()
    expect(renderHome({ reachable: REACHABLE })).toBeUndefined()
  })

  it('resolves the own organization', () => {
    expect(renderHome({ me, reachable: REACHABLE })).toBe('amt')
  })

  it('resolves the scope first', () => {
    setOrganizationScope('u1', 'nord')
    expect(renderHome({ me, reachable: REACHABLE })).toBe('nord')
  })

  it('needs no own organization in the auth bypass', () => {
    bypass.value = true
    expect(renderHome({ reachable: REACHABLE })).toBe('betrieb')
  })
})
