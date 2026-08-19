import type { OrganizationResponse } from '@/api/backendApi'

export interface OrgNode {
  org: OrganizationResponse
  children: OrgNode[]
}

const byName = (a: OrgNode, b: OrgNode): number => a.org.name.localeCompare(b.org.name, 'de')

/**
 * Builds the subtree below `rootId`.
 *
 * Two malformed inputs are tolerated rather than trusted: a node whose parent is
 * absent from `orgs` is unreachable and gets dropped instead of being silently
 * re-parented, and a parent chain that loops back on itself is cut at the repeat
 * so the recursion cannot run forever.
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

export const nodeOf = (root: OrgNode, orgId: string): OrgNode | null => {
  if (root.org.id === orgId) return root
  for (const child of root.children) {
    const found = nodeOf(child, orgId)
    if (found) return found
  }
  return null
}

export const flatten = (node: OrgNode): OrganizationResponse[] => [
  node.org,
  ...node.children.flatMap(flatten),
]
