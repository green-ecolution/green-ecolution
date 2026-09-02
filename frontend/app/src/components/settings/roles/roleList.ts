import type { TFunction } from 'i18next'
import type { Role } from '@/api/backendApi'

// `t`'s generated overloads only accept the catalog's literal key union; a key
// built from the backend's runtime template_key can't satisfy that statically.
type LooseTranslate = (key: string) => string

export const samePermissionSet = (a: readonly string[], b: readonly string[]): boolean => {
  const left = new Set(a)
  const right = new Set(b)
  return left.size === right.size && [...left].every((entry) => right.has(entry))
}

/**
 * The migration seeds one editable copy per template, sharing its name and
 * permissions. Listing both would show every name twice, so an untouched copy
 * stays hidden until someone renames it or changes its rights.
 *
 * The backend records that state as `template_key`, which it clears on the
 * first edit, so the copy no longer has to be recognised by comparing names.
 */
export const isPristineTemplateCopy = (role: Role): boolean =>
  !role.isTemplate && role.templateKey != null

export const ownRolesOf = (roles: Role[]): Role[] =>
  roles.filter((role) => !role.isTemplate && !isPristineTemplateCopy(role))

/**
 * Names of the delivered roles, keyed by the backend's `template_key`, resolved
 * through settings:roles.templateName.* so the display name follows the active
 * language. These live here rather than in the database because a name shipped
 * by a migration cannot be translated. The moment a user edits the role the
 * backend drops the key, and the stored name wins again.
 */
const TEMPLATE_KEYS: readonly string[] = [
  'administrator',
  'tree_care',
  'sensors',
  'route_planning',
  'observer',
]

/** Label for a role: the catalog name while untouched, the stored name after. */
export const roleDisplayName = (
  role: Pick<Role, 'name' | 'templateKey'>,
  t: TFunction<'settings'>,
): string => {
  const key = role.templateKey
  if (key == null || !TEMPLATE_KEYS.includes(key)) return role.name
  return (t as LooseTranslate)(`roles.templateName.${key}`)
}
