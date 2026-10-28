import type {
  ModuleListResponse,
  OrganizationResourceCountsResponse,
  OrganizationResponse,
} from '@green-ecolution/backend-client'
import {
  buildTree,
  flatten,
  subtreeMemberCount,
} from '@/components/settings/organization/organizationTree'

export interface TenantRow {
  id: string
  name: string
  subOrganizationCount: number
  memberCount: number
  /** `null` while the module list has not arrived; the card shows no badge, not "none". */
  activeModules: string[] | null
  /** `null` while the counts have not arrived; the card shows a placeholder, not 0. */
  sensorCount: number | null
  vehicleCount: number | null
  planCount: number | null
}

/**
 * One row per direct child of the root, with everything below it folded in.
 * `null` while the list holds no root, e.g. a refetch under a tenant scope.
 */
export function tenantRows(
  organizations: OrganizationResponse[],
  modules: ModuleListResponse | undefined,
  counts: OrganizationResourceCountsResponse[] | undefined,
): TenantRow[] | null {
  const root = organizations.find((org) => org.parentId == null)
  const tree = root ? buildTree(organizations, root.id) : null
  if (!tree) return null

  const activeByOrg = new Map(
    (modules?.organizations ?? []).map((entry) => [entry.organizationId, entry.active]),
  )
  const countsByOrg = new Map((counts ?? []).map((entry) => [entry.organizationId, entry]))

  return tree.children.map((node) => {
    const resources = countsByOrg.get(node.org.id)
    return {
      id: node.org.id,
      name: node.org.name,
      subOrganizationCount: flatten(node).length - 1,
      memberCount: subtreeMemberCount(node),
      activeModules: modules ? (activeByOrg.get(node.org.id) ?? []) : null,
      sensorCount: resources?.sensorCount ?? null,
      vehicleCount: resources?.vehicleCount ?? null,
      planCount: resources?.planCount ?? null,
    }
  })
}
