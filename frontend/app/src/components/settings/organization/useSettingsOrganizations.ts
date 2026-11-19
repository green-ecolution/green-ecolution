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
  /**
   * The entries from GET /organizations. Only these carry a real member count
   * and contact person; the others stand in with `memberCount: 0`.
   */
  readableIds: ReadonlySet<string>
}

const NONE: ReadonlySet<string> = new Set()

/**
 * The reachable list ignores the scope, so it is cut down to the scope subtree:
 * requests for an organization outside it would be rejected.
 */
const reachableOutside = (
  reachable: ReachableOrganizationResponse[],
  scope: string | null,
  known: ReadonlySet<string>,
): OrganizationResponse[] => {
  const parentOf = new Map(reachable.map((org) => [org.id, org.parentId]))
  return reachable
    .filter((org) => !known.has(org.id))
    .filter((org) => !scope || isAncestorOrSelf(scope, org.id, parentOf))
    .map((org) => ({ id: org.id, name: org.name, parentId: org.parentId, memberCount: 0 }))
}

/**
 * The organizations the settings offer: GET /organizations, completed by every
 * reachable organization it lacks. A role may grant, say, start_point:read in
 * an organization without organization:read there, and that organization must
 * still be selectable and part of its subtree.
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
  // The bypass reads every organization already.
  const { data: reachable } = useQuery({
    ...userQueries.myOrganizations(),
    enabled: rolesKnown && !bypass,
  })

  if (!rolesKnown) return { organizations: undefined, readableIds: NONE }
  const readable = canReadOrganizations ? visible : []
  if (!readable || (!bypass && !reachable)) return { organizations: undefined, readableIds: NONE }

  const readableIds = new Set(readable.map((org) => org.id))
  return {
    organizations: [...readable, ...reachableOutside(reachable ?? [], scope, readableIds)],
    readableIds,
  }
}
