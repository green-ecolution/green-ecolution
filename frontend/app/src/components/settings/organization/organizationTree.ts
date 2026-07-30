import type { OrganizationResponse } from '@/api/backendApi'

export interface OrgNode {
  org: OrganizationResponse
  children: OrgNode[]
}

const byName = (a: OrgNode, b: OrgNode): number => a.org.name.localeCompare(b.org.name, 'de')

/** The tile glyph shared by the tree rows and the detail header. */
export const initialsOf = (name: string): string =>
  name
    .trim()
    .split(/\s+/)
    .slice(0, 2)
    .map((word) => word[0]?.toUpperCase() ?? '')
    .join('')

/**
 * Builds the subtree below `rootId`. Nodes whose parent is not in `orgs` are
 * unreachable and therefore dropped rather than silently re-parented.
 */
export const buildTree = (orgs: OrganizationResponse[], rootId: string): OrgNode | null => {
  const root = orgs.find((o) => o.id === rootId)
  if (!root) return null

  const childrenOf = new Map<string, OrganizationResponse[]>()
  for (const org of orgs) {
    if (org.parentId === null || org.parentId === undefined) continue
    const siblings = childrenOf.get(org.parentId) ?? []
    siblings.push(org)
    childrenOf.set(org.parentId, siblings)
  }

  const build = (org: OrganizationResponse, visited: ReadonlySet<string>): OrgNode => {
    const seen = new Set(visited).add(org.id)
    return {
      org,
      children: (childrenOf.get(org.id) ?? [])
        .filter((child) => !seen.has(child.id))
        .map((child) => build(child, seen))
        .sort(byName),
    }
  }

  return build(root, new Set())
}

export const subtreeMemberCount = (node: OrgNode): number =>
  node.org.memberCount + node.children.reduce((sum, child) => sum + subtreeMemberCount(child), 0)

export const pathTo = (root: OrgNode, orgId: string): OrganizationResponse[] => {
  if (root.org.id === orgId) return [root.org]
  for (const child of root.children) {
    const tail = pathTo(child, orgId)
    if (tail.length > 0) return [root.org, ...tail]
  }
  return []
}

export const flatten = (node: OrgNode): OrganizationResponse[] => [
  node.org,
  ...node.children.flatMap(flatten),
]
