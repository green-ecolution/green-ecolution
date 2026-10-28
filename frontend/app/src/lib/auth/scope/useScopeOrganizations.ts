import { useQuery } from '@tanstack/react-query'
import type { ReachableOrganizationResponse } from '@green-ecolution/backend-client'
import { userQueries } from '@/api/queries'
import { useOrganizationScope } from './organizationScopeStore'

export interface ScopeOrganizations {
  organizations: ReachableOrganizationResponse[]
  /** More than one reachable organization; otherwise there is nothing to switch. */
  switchable: boolean
  activeName: string | null
  scope: string | null
}

export function useScopeOrganizations(): ScopeOrganizations {
  const scope = useOrganizationScope()
  const { data } = useQuery(userQueries.myOrganizations())
  const organizations = data ?? []
  const switchable = organizations.length > 1
  const activeName =
    switchable && scope ? (organizations.find((o) => o.id === scope)?.name ?? null) : null
  return { organizations, switchable, activeName, scope }
}
