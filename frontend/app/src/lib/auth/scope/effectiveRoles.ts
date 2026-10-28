import { useQuery } from '@tanstack/react-query'
import type { QueryClient } from '@tanstack/react-query'
import type { ReachableOrganizationResponse, RoleResponse } from '@green-ecolution/backend-client'
import { userQueries } from '@/api/queries'
import { useAuthSession } from '../authSessionContext'
import { readAuthBypass } from '../runtimeConfig'
import { getOrganizationScope, useOrganizationScope } from './organizationScopeStore'

const isAncestorOrSelf = (
  ancestorId: string,
  orgId: string,
  parentOf: ReadonlyMap<string, string | null | undefined>,
): boolean => {
  const seen = new Set<string>()
  for (let id: string | null | undefined = orgId; id && !seen.has(id); id = parentOf.get(id)) {
    if (id === ancestorId) return true
    seen.add(id)
  }
  return false
}

/**
 * The roles as they apply under the selected scope. A role held above the
 * scope grants its rights for the scope organization (its subtree covers it),
 * a role below stays as is, an unrelated one cannot act inside the scope.
 */
export function effectiveRoles(
  roles: RoleResponse[],
  scope: string | null,
  reachable: ReachableOrganizationResponse[],
): RoleResponse[] {
  if (!scope) return roles

  const parentOf = new Map(reachable.map((org) => [org.id, org.parentId]))
  return roles.flatMap((role) => {
    if (!role.organizationId) return []
    if (isAncestorOrSelf(role.organizationId, scope, parentOf)) {
      return [{ ...role, organizationId: scope }]
    }
    return isAncestorOrSelf(scope, role.organizationId, parentOf) ? [role] : []
  })
}

export async function ensureEffectiveRoles(queryClient: QueryClient): Promise<RoleResponse[]> {
  const me = await queryClient.ensureQueryData(userQueries.me())
  const scope = getOrganizationScope()
  if (!scope) return me.roles
  const reachable = await queryClient.ensureQueryData(userQueries.myOrganizations())
  return effectiveRoles(me.roles, scope, reachable)
}

/** `undefined` until the data needed to apply the scope has loaded. */
export function useEffectiveRoles(): RoleResponse[] | undefined {
  const { isAuthenticated } = useAuthSession()
  const enabled = isAuthenticated && !readAuthBypass()
  const scope = useOrganizationScope()
  const { data: me } = useQuery({ ...userQueries.me(), enabled })
  const { data: reachable } = useQuery({
    ...userQueries.myOrganizations(),
    enabled: enabled && scope !== null,
  })

  if (!me) return undefined
  if (!scope) return me.roles
  return reachable ? effectiveRoles(me.roles, scope, reachable) : undefined
}

/** The organization the user acts from: the scope, else their own. */
export function useHomeOrganizationId(): string | null {
  const scope = useOrganizationScope()
  const { data: me } = useQuery(userQueries.me())
  return scope ?? me?.organization?.id ?? null
}
