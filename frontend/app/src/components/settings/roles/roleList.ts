import type { Role } from '@/api/backendApi'

export const samePermissionSet = (a: readonly string[], b: readonly string[]): boolean => {
  const left = new Set(a)
  const right = new Set(b)
  return left.size === right.size && [...left].every((entry) => right.has(entry))
}

/**
 * The migration seeds one editable copy per template, sharing its name and
 * permissions. Listing both would show every name twice, so an untouched copy
 * stays hidden until someone renames it or changes its rights.
 */
export const isPristineTemplateCopy = (role: Role, templates: Role[]): boolean =>
  templates.some(
    (template) =>
      template.name === role.name && samePermissionSet(template.permissions, role.permissions),
  )

export const ownRolesOf = (roles: Role[], templates: Role[]): Role[] =>
  roles.filter((role) => !role.isTemplate && !isPristineTemplateCopy(role, templates))
