import type {
  ModuleReleaseResponse,
  ModuleReleasesResponse,
  OrganizationResourceCountsResponse,
  OrganizationResponse,
} from '@green-ecolution/backend-client'
import {
  buildTree,
  flatten,
  subtreeMemberCount,
} from '@/components/settings/organization/organizationTree'

export type TenantModuleCell = ModuleReleaseResponse

export interface TenantRow {
  id: string
  name: string
  subOrganizationCount: number
  memberCount: number
  /** `null` while the releases have not arrived or cannot be read; the table then has no module columns. */
  modules: TenantModuleCell[] | null
  /** `null` while the counts have not arrived; the table shows a placeholder, not 0. */
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
  releases: ModuleReleasesResponse | undefined,
  counts: OrganizationResourceCountsResponse[] | undefined,
): TenantRow[] | null {
  const root = organizations.find((org) => org.parentId == null)
  const tree = root ? buildTree(organizations, root.id) : null
  if (!tree) return null

  const modulesByOrg = new Map(
    (releases?.tenants ?? []).map((entry) => [entry.organizationId, entry.modules]),
  )
  const countsByOrg = new Map((counts ?? []).map((entry) => [entry.organizationId, entry]))

  return tree.children.map((node) => {
    const resources = countsByOrg.get(node.org.id)
    return {
      id: node.org.id,
      name: node.org.name,
      subOrganizationCount: flatten(node).length - 1,
      memberCount: subtreeMemberCount(node),
      modules: releases ? (modulesByOrg.get(node.org.id) ?? []) : null,
      sensorCount: resources?.sensorCount ?? null,
      vehicleCount: resources?.vehicleCount ?? null,
      planCount: resources?.planCount ?? null,
    }
  })
}
