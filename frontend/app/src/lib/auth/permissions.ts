import type { RoleResponse } from '@green-ecolution/backend-client'
import type { ModuleManifest } from '@/modules/types'

export const CORE_RESOURCES = [
  'sensor',
  'start_point',
  'vehicle',
  'region',
  'user',
  'organization',
  'role',
  'plugin',
  'setting',
] as const

/**
 * Modules add the keys of the resources they declare by augmenting this
 * interface, so their permission strings stay checked at compile time without
 * the core naming them.
 */
// eslint-disable-next-line @typescript-eslint/no-empty-object-type
export interface ModuleResourceKeys {}

export const ACTIONS = ['read', 'create', 'update', 'delete'] as const

export type Resource = (typeof CORE_RESOURCES)[number] | keyof ModuleResourceKeys
export type Action = (typeof ACTIONS)[number]
export type Permission = `${Resource}:${Action}`

/** OR semantics; an empty requirement is satisfied by everyone. */
export type PermissionRequirement = readonly Permission[]

export const UNRESTRICTED = 'unrestricted'

/** A plain string set, so a permission the backend adds later cannot break parsing. */
export type Permissions = ReadonlySet<string> | typeof UNRESTRICTED

export const NO_PERMISSIONS: Permissions = new Set<string>()

export function resourcesFor(manifests: readonly ModuleManifest[]): Resource[] {
  return [...CORE_RESOURCES, ...manifests.flatMap((m) => (m.resources ?? []).map((r) => r.key))]
}

export const readPermissionsOf = (resources: readonly Resource[]): Permission[] =>
  resources.map((resource): Permission => `${resource}:read`)

/**
 * Union of every grant the user holds. Org scoping is deliberately dropped:
 * nav and route gating only ask whether a right is held anywhere, the backend
 * enforces the organization subtree.
 */
export function permissionsOf(roles: RoleResponse[] | undefined): Permissions {
  if (!roles) return NO_PERMISSIONS
  return new Set(roles.flatMap((role) => role.permissions))
}

export function satisfies(perms: Permissions, required: PermissionRequirement): boolean {
  if (required.length === 0) return true
  if (perms === UNRESTRICTED) return true
  return required.some((permission) => perms.has(permission))
}
