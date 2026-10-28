import { useQuery } from '@tanstack/react-query'
import type { ReachableOrganizationResponse } from '@green-ecolution/backend-client'
import { userQueries } from '@/api/queries'
import { readAuthBypass } from '@/lib/auth/runtimeConfig'
import { buildOrganizationRows } from './organizationRows'
import { useOrganizationScope } from './organizationScopeStore'

export interface ScopeOrganizations {
  organizations: ReachableOrganizationResponse[]
  /** More than one reachable organization; otherwise there is nothing to switch. */
  switchable: boolean
  /** The organization that stands for "no scope": the user's own, else the topmost reachable one. */
  homeId: string | null
  /** The scope, or the home organization while no scope is set. */
  activeId: string | null
  activeName: string | null
  scope: string | null
}

export function useScopeOrganizations(): ScopeOrganizations {
  const scope = useOrganizationScope()
  const { data } = useQuery(userQueries.myOrganizations())
  const { data: me } = useQuery({ ...userQueries.me(), enabled: !readAuthBypass() })
  const organizations = data ?? []
  const switchable = organizations.length > 1

  const ownId = me?.organization?.id
  const homeId = organizations.some((org) => org.id === ownId)
    ? (ownId ?? null)
    : (buildOrganizationRows(organizations)[0]?.id ?? null)
  const activeId = scope ?? homeId
  const activeName = switchable
    ? (organizations.find((org) => org.id === activeId)?.name ?? null)
    : null

  return { organizations, switchable, homeId, activeId, activeName, scope }
}
