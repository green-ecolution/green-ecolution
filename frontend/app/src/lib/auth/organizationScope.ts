import type { OrganizationResponse, RoleResponse } from '@green-ecolution/backend-client'
import { buildTree, flatten } from '@/components/settings/organization/organizationTree'
import type { Permission } from './permissions'

export const byOrgName = (a: OrganizationResponse, b: OrganizationResponse): number =>
  a.name.localeCompare(b.name, 'de')

/**
 * Organizations in which the user holds `permission`.
 *
 * A role grants its permissions for the owning organization *and* its whole
 * subtree, so each matching role contributes the subtree below its own org.
 * Roles without an organization are templates: delivered by migration, never
 * assignable, and therefore no grant.
 */
export function orgsWithPermission(
  orgs: OrganizationResponse[],
  roles: RoleResponse[],
  permission: Permission,
): OrganizationResponse[] {
  const granted = new Map<string, OrganizationResponse>()

  for (const role of roles) {
    if (!role.organizationId) continue
    if (!role.permissions.includes(permission)) continue

    const subtree = buildTree(orgs, role.organizationId)
    if (!subtree) continue
    for (const org of flatten(subtree)) granted.set(org.id, org)
  }

  return [...granted.values()].sort(byOrgName)
}
