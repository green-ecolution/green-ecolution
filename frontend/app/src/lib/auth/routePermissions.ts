import {
  satisfies,
  type Action,
  type Permission,
  type PermissionRequirement,
  type Permissions,
  type Resource,
} from './permissions'
import type { ModuleManifest } from '@/modules/types'

export type RoutePermissions = Readonly<Record<string, PermissionRequirement>>

/**
 * Core part of the single source of truth for nav filtering and route guards.
 * Paths without an entry are open: /dashboard, /settings, /settings/profile,
 * /info, /debug and /help. /settings/organization and /admin are listed
 * because they cannot render without their read right. The organization's sub-pages are
 * filtered per selected organization by settingsNav. Module routes come
 * from the manifests.
 */
export const CORE_ROUTE_PERMISSIONS: RoutePermissions = {
  '/admin': ['organization:read'],
  '/sensors': ['sensor:read'],
  '/vehicles': ['vehicle:read'],
  '/settings/organization': ['organization:read'],
  // Only the instance navigation links these directly; inside the settings
  // the entries are filtered per selected organization by settingsNav.
  '/settings/organization/$orgId': ['organization:read'],
  '/settings/organization/$orgId/members': ['user:read'],
  '/settings/organization/$orgId/plugins': ['plugin:read'],
}

/** Any one module's map right opens the map, since each module draws only its own layers. */
export function mapPermissionsFor(manifests: readonly ModuleManifest[]): PermissionRequirement {
  return [...new Set(manifests.flatMap(({ map }) => map?.permissions ?? []))]
}

/** The resources guarding the plans of every module that can plan. */
export function planResourcesOf(manifests: readonly ModuleManifest[]): Resource[] {
  return manifests.flatMap(({ planning }) => (planning ? [planning.planResource] : []))
}

/** Holding the action on any one module's plans is enough; the backend checks the plan's own module. */
export function planPermissionsFor(
  manifests: readonly ModuleManifest[],
  action: Action,
): Permission[] {
  return planResourcesOf(manifests).map((resource): Permission => `${resource}:${action}`)
}

export function routePermissionsFor(manifests: readonly ModuleManifest[]): RoutePermissions {
  const planPermissions = planPermissionsFor(manifests, 'read')
  const mapPermissions = mapPermissionsFor(manifests)
  return {
    ...CORE_ROUTE_PERMISSIONS,
    ...(mapPermissions.length > 0 ? { '/map': mapPermissions } : {}),
    ...(planPermissions.length > 0 ? { '/plans': planPermissions } : {}),
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
