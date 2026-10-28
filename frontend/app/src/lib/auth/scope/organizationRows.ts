import type { ReachableOrganizationResponse } from '@green-ecolution/backend-client'

export interface OrganizationRow {
  id: string
  name: string
  depth: number
}

/**
 * The reachable organizations as an indented, name-sorted tree. Roots are the
 * entries whose parent is not in the list. With a query, only matching entries
 * remain, together with their ancestors for context.
 */
export function buildOrganizationRows(
  organizations: ReachableOrganizationResponse[],
  query = '',
): OrganizationRow[] {
  const byId = new Map(organizations.map((org) => [org.id, org]))
  const children = new Map<string | null, ReachableOrganizationResponse[]>()
  for (const org of organizations) {
    const key = org.parentId && byId.has(org.parentId) ? org.parentId : null
    children.set(key, [...(children.get(key) ?? []), org])
  }

  const needle = query.trim().toLocaleLowerCase()
  const visible = new Set<string>()
  if (needle) {
    for (const org of organizations) {
      if (!org.name.toLocaleLowerCase().includes(needle)) continue
      for (
        let current: ReachableOrganizationResponse | undefined = org;
        current && !visible.has(current.id);
        current = current.parentId ? byId.get(current.parentId) : undefined
      ) {
        visible.add(current.id)
      }
    }
  }

  const rows: OrganizationRow[] = []
  const seen = new Set<string>()
  const walk = (parent: string | null, depth: number) => {
    const level = [...(children.get(parent) ?? [])].sort((a, b) => a.name.localeCompare(b.name))
    for (const org of level) {
      if (seen.has(org.id) || (needle && !visible.has(org.id))) continue
      seen.add(org.id)
      rows.push({ id: org.id, name: org.name, depth })
      walk(org.id, depth + 1)
    }
  }
  walk(null, 0)
  return rows
}
