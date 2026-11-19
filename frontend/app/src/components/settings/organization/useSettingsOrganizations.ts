import { useQuery } from '@tanstack/react-query'
import type { OrganizationResponse, ReachableOrganizationResponse } from '@/api/backendApi'
import { organizationQueries, userQueries } from '@/api/queries'
import { permissionsOf, satisfies } from '@/lib/auth/permissions'
import { readAuthBypass } from '@/lib/auth/runtimeConfig'
import { isAncestorOrSelf, useEffectiveRoles } from '@/lib/auth/scope/effectiveRoles'
import { useOrganizationScope } from '@/lib/auth/scope/organizationScopeStore'

export interface SettingsOrganizationList {
  /** `undefined` while loading. */
  organizations: OrganizationResponse[] | undefined
  /** Only GET /organizations carries real member counts; the fallback reports 0. */
  hasMemberCounts: boolean
}

/**
 * The reachable list ignores the scope, so it is cut down to the scope subtree:
 * requests for an organization outside it would be rejected.
 */
const fromReachable = (
  reachable: ReachableOrganizationResponse[],
  scope: string | null,
): OrganizationResponse[] => {
  const parentOf = new Map(reachable.map((org) => [org.id, org.parentId]))
  return reachable
    .filter((org) => !scope || isAncestorOrSelf(scope, org.id, parentOf))
    .map((org) => ({ id: org.id, name: org.name, parentId: org.parentId, memberCount: 0 }))
}

/**
 * The organizations the settings offer. GET /organizations needs
 * organization:read; without it anywhere, the reachable organizations stand
 * in, so pages such as stations work for roles that only hold their own right.
 */
export function useSettingsOrganizations(): SettingsOrganizationList {
  const bypass = readAuthBypass()
  const roles = useEffectiveRoles()
  const scope = useOrganizationScope()
  const rolesKnown = bypass || roles !== undefined
  const canReadOrganizations = bypass || satisfies(permissionsOf(roles), ['organization:read'])

  const { data: visible } = useQuery({
    ...organizationQueries.list(),
    enabled: rolesKnown && canReadOrganizations,
  })
  const { data: reachable } = useQuery({
    ...userQueries.myOrganizations(),
    enabled: rolesKnown && !canReadOrganizations,
  })

  if (!rolesKnown) return { organizations: undefined, hasMemberCounts: false }
  if (canReadOrganizations) return { organizations: visible, hasMemberCounts: true }
  return {
    organizations: reachable && fromReachable(reachable, scope),
    hasMemberCounts: false,
  }
}
