import { ANY_READ, satisfies, type PermissionRequirement, type Permissions } from './permissions'

/**
 * Single source of truth for nav filtering and route guards. Paths without an
 * entry are open, which covers /dashboard, /settings, /settings/profile, /info and /debug.
 */
export const ROUTE_PERMISSIONS: Record<string, PermissionRequirement> = {
  '/trees': ['tree:read'],
  '/treecluster': ['tree_cluster:read'],
  '/sensors': ['sensor:read'],
  '/watering-plans': ['watering_plan:read'],
  '/vehicles': ['vehicle:read'],
  '/settings/team/members': ['user:read'],
  '/settings/team/roles': ['role:read'],
  '/map': ['tree:read', 'tree_cluster:read'],
  '/evaluations': ANY_READ,
}

// `to` on nav entries carries TanStack's wide LinkProps type, hence unknown.
export function canAccessRoute(path: unknown, perms: Permissions): boolean {
  if (typeof path !== 'string') return true
  const required: PermissionRequirement | undefined = ROUTE_PERMISSIONS[path]
  return required === undefined || satisfies(perms, required)
}

interface NavSectionLike {
  links: readonly { to?: unknown }[]
}

export function visibleNavSections<S extends NavSectionLike>(
  sections: readonly S[],
  perms: Permissions,
): S[] {
  return sections
    .map((section) => ({
      ...section,
      links: section.links.filter((link) => canAccessRoute(link.to, perms)),
    }))
    .filter((section) => section.links.length > 0)
}
