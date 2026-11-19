import type { OrganizationResponse } from '@/api/backendApi'
import { buildOrganizationRows } from '@/lib/auth/scope/organizationRows'

export interface StructureRow {
  id: string
  name: string
  depth: number
  /** Members of the organization and everything below it. */
  memberTotal: number
  memberDirect: number
  contactPersonId: string | null
  moduleKeys: string[]
}

export function structureRows(
  organizations: OrganizationResponse[],
  activeByOrg: ReadonlyMap<string, readonly string[]>,
): StructureRow[] {
  const byId = new Map(organizations.map((org) => [org.id, org]))
  const childrenOf = new Map<string, OrganizationResponse[]>()
  for (const org of organizations) {
    if (!org.parentId || !byId.has(org.parentId)) continue
    childrenOf.set(org.parentId, [...(childrenOf.get(org.parentId) ?? []), org])
  }
  const totalOf = (org: OrganizationResponse): number =>
    org.memberCount + (childrenOf.get(org.id) ?? []).reduce((sum, child) => sum + totalOf(child), 0)

  return buildOrganizationRows(organizations).flatMap((row) => {
    const org = byId.get(row.id)
    if (!org) return []
    return [
      {
        id: org.id,
        name: org.name,
        depth: row.depth,
        memberTotal: totalOf(org),
        memberDirect: org.memberCount,
        contactPersonId: org.contactPersonId ?? null,
        moduleKeys: [...(activeByOrg.get(org.id) ?? [])],
      },
    ]
  })
}
