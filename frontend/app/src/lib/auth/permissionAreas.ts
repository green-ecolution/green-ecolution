import type { TFunction } from 'i18next'
import {
  ACTIONS,
  RESOURCES,
  UNRESTRICTED,
  type Action,
  type Permission,
  type Permissions,
  type Resource,
} from './permissions'

export const ACCESS_LEVELS = ['none', 'view', 'edit', 'manage'] as const
export type AccessLevel = (typeof ACCESS_LEVELS)[number]

/**
 * Presets only. The stored truth is always the individual action set, so a role
 * whose actions match no preset stays valid and reads as "custom".
 */
const LEVEL_ACTIONS: Record<AccessLevel, readonly Action[]> = {
  none: [],
  view: ['read'],
  edit: ['read', 'create', 'update'],
  manage: ['read', 'create', 'update', 'delete'],
}

export type AreaGroup = 'greenspaces' | 'operations' | 'administration'

export const AREA_GROUP_ORDER: readonly AreaGroup[] = [
  'greenspaces',
  'operations',
  'administration',
]

export interface AreaAction {
  action: Action
  permission: Permission
  label: string
  hint: string
}

export interface PermissionArea {
  resource: Resource
  group: AreaGroup
  label: string
  description: string
  actions: AreaAction[]
}

interface AreaDefinition {
  resource: Resource
  group: AreaGroup
}

const AREA_DEFINITIONS: readonly AreaDefinition[] = [
  { resource: 'tree', group: 'greenspaces' },
  { resource: 'tree_cluster', group: 'greenspaces' },
  { resource: 'sensor', group: 'greenspaces' },
  { resource: 'region', group: 'greenspaces' },
  { resource: 'watering_plan', group: 'operations' },
  { resource: 'vehicle', group: 'operations' },
  { resource: 'user', group: 'administration' },
  { resource: 'organization', group: 'administration' },
  { resource: 'role', group: 'administration' },
]

const permissionFor = (resource: Resource, action: Action): Permission => `${resource}:${action}`

/**
 * Composing "{resource} {verb}" from the four generic action verbs is lossless
 * for 34 of the 36 cells, but two lose a real distinction the generic verb
 * can't carry: inviting a person is not creating one, and removing their
 * access is not deleting them. These two literal overrides take priority
 * over the composed form; everything else still composes — including
 * watering_plan:create, where the composed "Einsatzplan anlegen" already says
 * everything "planen" would have: the specificity in the old "Einsatz planen"
 * lived in the noun ("Einsatz" denoted the activity itself), and once the
 * entity is correctly named "Einsatzplan" the extra verb adds nothing.
 */
const actionLabelOverride = (
  resource: Resource,
  action: Action,
  t: TFunction<'settings'>,
): string | null => {
  if (resource === 'user' && action === 'create') {
    return t('permission.resource.user.actionOverrides.create')
  }
  if (resource === 'user' && action === 'delete') {
    return t('permission.resource.user.actionOverrides.delete')
  }
  return null
}

export const levelLabels = (t: TFunction<'settings'>): Record<AccessLevel, string> => ({
  none: t('permission.level.none'),
  view: t('permission.level.view'),
  edit: t('permission.level.edit'),
  manage: t('permission.level.manage'),
})

export const areaGroupLabels = (t: TFunction<'settings'>): Record<AreaGroup, string> => ({
  greenspaces: t('permission.group.greenspaces'),
  operations: t('permission.group.operations'),
  administration: t('permission.group.administration'),
})

/**
 * Composes the nine areas and their 36 actions from settings:permission.resource.*
 * (label, singular noun, description, and the per-action hint, which genuinely
 * differs per permission) and settings:permission.action.* (the four verb
 * templates, "read" filled with the plural label, the rest with the singular
 * noun) — with two explicit label overrides (see actionLabelOverride) for
 * the cells the generic verbs flatten. Not a component: the caller passes its
 * own scoped `t`, mirroring clusterStatusReason.ts.
 */
export const permissionAreasFor = (t: TFunction<'settings'>): PermissionArea[] =>
  AREA_DEFINITIONS.map((definition) => {
    const resourceKey = `permission.resource.${definition.resource}` as const
    const singular = t(`${resourceKey}.singular`)
    const pluralNoun = t(`${resourceKey}.pluralNoun`)
    return {
      resource: definition.resource,
      group: definition.group,
      label: t(`${resourceKey}.label`),
      description: t(`${resourceKey}.description`),
      actions: ACTIONS.map((action) => ({
        action,
        permission: permissionFor(definition.resource, action),
        label:
          actionLabelOverride(definition.resource, action, t) ??
          t(`permission.action.${action}`, {
            resource: action === 'read' ? pluralNoun : singular,
          }),
        hint: t(`${resourceKey}.hint.${action}`),
      })),
    }
  })

export const activeActionsOf = (resource: Resource, perms: ReadonlySet<string>): Action[] =>
  ACTIONS.filter((action) => perms.has(permissionFor(resource, action)))

export const activeActionCount = (resource: Resource, perms: ReadonlySet<string>): number =>
  activeActionsOf(resource, perms).length

export const levelOf = (resource: Resource, perms: ReadonlySet<string>): AccessLevel | 'custom' => {
  const active = activeActionsOf(resource, perms)
  const match = ACCESS_LEVELS.find(
    (level) =>
      LEVEL_ACTIONS[level].length === active.length &&
      LEVEL_ACTIONS[level].every((action) => active.includes(action)),
  )
  return match ?? 'custom'
}

export const applyLevel = (
  perms: ReadonlySet<string>,
  resource: Resource,
  level: AccessLevel,
): Set<string> => {
  const next = new Set(perms)
  ACTIONS.forEach((action) => next.delete(permissionFor(resource, action)))
  LEVEL_ACTIONS[level].forEach((action) => next.add(permissionFor(resource, action)))
  return next
}

/**
 * Applies a preset but only touches actions the caller may grant. Non-grantable
 * actions are left exactly as they were, so a preset can neither add a right the
 * caller lacks nor silently strip a pre-existing frozen one.
 */
export const applyLevelWithinGrantable = (
  perms: ReadonlySet<string>,
  resource: Resource,
  level: AccessLevel,
  grantable: Permissions,
): Set<string> => {
  const next = new Set(perms)
  const preset = new Set(LEVEL_ACTIONS[level])
  ACTIONS.forEach((action) => {
    const permission = permissionFor(resource, action)
    if (!isGrantable(permission, grantable)) return
    if (preset.has(action)) next.add(permission)
    else next.delete(permission)
  })
  return next
}

export const toggleAction = (perms: ReadonlySet<string>, permission: Permission): Set<string> => {
  const next = new Set(perms)
  if (!next.delete(permission)) next.add(permission)
  return next
}

const KNOWN_PERMISSIONS: ReadonlySet<string> = new Set(
  RESOURCES.flatMap((resource) => ACTIONS.map((action) => permissionFor(resource, action))),
)

/**
 * PATCH replaces the whole permission set, so anything the backend adds later
 * must survive an edit untouched instead of being silently dropped.
 */
export const unknownPermissions = (perms: ReadonlySet<string>): string[] =>
  [...perms].filter((permission) => !KNOWN_PERMISSIONS.has(permission)).sort()

export const isGrantable = (permission: string, grantable: Permissions): boolean =>
  grantable === UNRESTRICTED || grantable.has(permission)

export const clampToGrantable = (
  perms: ReadonlySet<string>,
  grantable: Permissions,
): { permissions: Set<string>; removed: string[] } => {
  if (grantable === UNRESTRICTED) return { permissions: new Set(perms), removed: [] }

  const permissions = new Set<string>()
  const removed: string[] = []
  perms.forEach((permission) => {
    if (grantable.has(permission)) permissions.add(permission)
    else removed.push(permission)
  })
  return { permissions, removed: removed.sort() }
}
