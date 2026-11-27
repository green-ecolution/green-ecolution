import type { TFunction } from 'i18next'
import type { Role } from '@/api/backendApi'
import { moduleManifests } from '@/modules/registry'
import { roleTemplateOf } from '@/modules/roleTemplates'

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
 * Label for a role: the catalog name of its template while untouched, the
 * stored name after. The names live in the catalogs rather than in the database
 * because a name shipped by a migration cannot be translated. The moment a
 * user edits the role the backend drops the template key, and the stored name
 * wins again.
 */
export const roleDisplayName = (
  role: Pick<Role, 'name' | 'templateKey'>,
  t: TFunction<'settings'>,
): string => {
  const template = roleTemplateOf(role.templateKey, moduleManifests)
  return template ? (t as LooseTranslate)(template.nameKey) : role.name
}
