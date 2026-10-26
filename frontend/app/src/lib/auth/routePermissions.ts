import {
  satisfies,
  type Permission,
  type PermissionRequirement,
  type Permissions,
  type Resource,
} from './permissions'
import type { ModuleManifest } from '@/modules/types'

export type RoutePermissions = Readonly<Record<string, PermissionRequirement>>

/**
 * Core part of the single source of truth for nav filtering and route guards.
 * Paths without an entry are open, which covers /dashboard, /settings,
 * /settings/profile, /info and /debug. Module routes come from the manifests.
 */
export const CORE_ROUTE_PERMISSIONS: RoutePermissions = {
  '/sensors': ['sensor:read'],
  '/vehicles': ['vehicle:read'],
  '/settings/team/members': ['user:read'],
  '/settings/team/roles': ['role:read'],
  '/map': ['tree:read', 'tree_cluster:read'],
}

/** The area `operations` holds the resources that guard operation plans. */
export function planResourcesOf(manifests: readonly ModuleManifest[]): Resource[] {
  return manifests.flatMap((manifest) =>
    (manifest.resources ?? []).filter((r) => r.area === 'operations').map((r) => r.key),
  )
}

export function routePermissionsFor(manifests: readonly ModuleManifest[]): RoutePermissions {
  const planPermissions = planResourcesOf(manifests).map(
    (resource): Permission => `${resource}:read`,
  )
  return {
    ...CORE_ROUTE_PERMISSIONS,
    ...(planPermissions.length > 0 ? { '/watering-plans': planPermissions } : {}),
    ...Object.fromEntries(
      manifests.flatMap((manifest) => Object.entries(manifest.routePermissions ?? {})),
    ),
  }
}

// `to` on nav entries carries TanStack's wide LinkProps type, hence unknown.
export function canAccessRoute(
  path: unknown,
  perms: Permissions,
  routePermissions: RoutePermissions,
): boolean {
  if (typeof path !== 'string') return true
  const required: PermissionRequirement | undefined = routePermissions[path]
  return required === undefined || satisfies(perms, required)
}

interface NavSectionLike {
  links: readonly { to?: unknown }[]
}

export function visibleNavSections<S extends NavSectionLike>(
  sections: readonly S[],
  perms: Permissions,
  routePermissions: RoutePermissions,
): S[] {
  return sections
    .map((section) => ({
      ...section,
      links: section.links.filter((link) => canAccessRoute(link.to, perms, routePermissions)),
    }))
    .filter((section) => section.links.length > 0)
}
