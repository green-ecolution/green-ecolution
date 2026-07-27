import type { UserResponse } from '@green-ecolution/backend-client'

export const RESOURCES = [
  'tree',
  'tree_cluster',
  'sensor',
  'watering_plan',
  'vehicle',
  'region',
  'user',
  'organization',
  'role',
] as const

export const ACTIONS = ['read', 'create', 'update', 'delete'] as const

export type Resource = (typeof RESOURCES)[number]
export type Action = (typeof ACTIONS)[number]
export type Permission = `${Resource}:${Action}`

/** OR semantics; an empty requirement is satisfied by everyone. */
export type PermissionRequirement = readonly Permission[]

export const UNRESTRICTED = 'unrestricted'

/** A plain string set, so a permission the backend adds later cannot break parsing. */
export type Permissions = ReadonlySet<string> | typeof UNRESTRICTED

export const NO_PERMISSIONS: Permissions = new Set<string>()

export const ANY_READ: PermissionRequirement = RESOURCES.map(
  (resource) => `${resource}:read` as Permission,
)

/**
 * Union of every grant the user holds. Org scoping is deliberately dropped:
 * nav and route gating only ask whether a right is held anywhere, the backend
 * enforces the organization subtree.
 */
export function permissionsOf(me: UserResponse | undefined): Permissions {
  if (!me) return NO_PERMISSIONS
  return new Set(me.roles.flatMap((role) => role.permissions))
}

export function satisfies(perms: Permissions, required: PermissionRequirement): boolean {
  if (required.length === 0) return true
  if (perms === UNRESTRICTED) return true
  return required.some((permission) => perms.has(permission))
}
