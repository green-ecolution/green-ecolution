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

export const LEVEL_LABELS: Record<AccessLevel, string> = {
  none: 'Kein',
  view: 'Ansehen',
  edit: 'Bearbeiten',
  manage: 'Verwalten',
}

/**
 * Presets only. The stored truth is always the individual action set, so a role
 * whose actions match no preset stays valid and reads as "Individuell".
 */
const LEVEL_ACTIONS: Record<AccessLevel, readonly Action[]> = {
  none: [],
  view: ['read'],
  edit: ['read', 'create', 'update'],
  manage: ['read', 'create', 'update', 'delete'],
}

export type AreaGroup = 'greenspaces' | 'operations' | 'administration'

export const AREA_GROUP_LABELS: Record<AreaGroup, string> = {
  greenspaces: 'Grünflächen',
  operations: 'Einsatzplanung',
  administration: 'Verwaltung',
}

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

type ActionCopy = Record<Action, readonly [label: string, hint: string]>

interface AreaDefinition {
  resource: Resource
  group: AreaGroup
  label: string
  description: string
  copy: ActionCopy
}

const AREA_DEFINITIONS: readonly AreaDefinition[] = [
  {
    resource: 'tree',
    group: 'greenspaces',
    label: 'Bäume',
    description: 'Bäume & Standorte',
    copy: {
      read: ['Bäume ansehen', 'Baumliste und Details einsehen'],
      create: ['Baum anlegen', 'Neue Bäume erfassen'],
      update: ['Baum bearbeiten', 'Stammdaten und Standort ändern'],
      delete: ['Baum löschen', 'Bäume dauerhaft entfernen'],
    },
  },
  {
    resource: 'tree_cluster',
    group: 'greenspaces',
    label: 'Bewässerungsgruppen',
    description: 'Gruppen & Schwellen',
    copy: {
      read: ['Gruppen ansehen', 'Gruppen und Bewässerungsstatus einsehen'],
      create: ['Gruppe anlegen', 'Neue Bewässerungsgruppen anlegen'],
      update: ['Gruppe bearbeiten', 'Zuordnung, Name und Schwellen ändern'],
      delete: ['Gruppe löschen', 'Gruppen auflösen'],
    },
  },
  {
    resource: 'sensor',
    group: 'greenspaces',
    label: 'Sensoren',
    description: 'Geräte & Netz',
    copy: {
      read: ['Sensoren ansehen', 'Geräte und Messwerte einsehen'],
      create: ['Sensor anlegen', 'Neues Gerät in Betrieb nehmen'],
      update: ['Sensor bearbeiten', 'Zuordnung und Stammdaten ändern'],
      delete: ['Sensor löschen', 'Gerät dauerhaft entfernen'],
    },
  },
  {
    resource: 'region',
    group: 'greenspaces',
    label: 'Gebiete',
    description: 'Stadtgebiete & Zuordnung',
    copy: {
      read: ['Gebiete ansehen', 'Stadtgebiete einsehen'],
      create: ['Gebiet anlegen', 'Neues Gebiet anlegen'],
      update: ['Gebiet bearbeiten', 'Gebiet umbenennen oder anpassen'],
      delete: ['Gebiet löschen', 'Gebiet entfernen'],
    },
  },
  {
    resource: 'watering_plan',
    group: 'operations',
    label: 'Einsätze',
    description: 'Planung & Touren',
    copy: {
      read: ['Einsätze ansehen', 'Geplante und abgeschlossene Einsätze einsehen'],
      create: ['Einsatz planen', 'Neuen Bewässerungseinsatz anlegen'],
      update: ['Einsatz bearbeiten', 'Route, Zeit, Umfang und Status ändern'],
      delete: ['Einsatz löschen', 'Einsatz dauerhaft entfernen'],
    },
  },
  {
    resource: 'vehicle',
    group: 'operations',
    label: 'Fahrzeuge',
    description: 'Transporter & Anhänger',
    copy: {
      read: ['Fahrzeuge ansehen', 'Fahrzeuge und Verfügbarkeit einsehen'],
      create: ['Fahrzeug anlegen', 'Neues Fahrzeug aufnehmen'],
      update: ['Fahrzeug bearbeiten', 'Stammdaten und Verfügbarkeit ändern'],
      delete: ['Fahrzeug löschen', 'Fahrzeug archivieren oder entfernen'],
    },
  },
  {
    resource: 'user',
    group: 'administration',
    label: 'Mitarbeitende',
    description: 'Konten & Zuordnung',
    copy: {
      read: ['Mitarbeitende ansehen', 'Mitarbeitende und ihre Rollen einsehen'],
      create: ['Mitarbeitende einladen', 'Neue Mitarbeitende einladen'],
      update: ['Mitarbeitende bearbeiten', 'Stammdaten und Rollenzuordnung ändern'],
      delete: ['Mitarbeitende entfernen', 'Zugang entziehen'],
    },
  },
  {
    resource: 'organization',
    group: 'administration',
    label: 'Organisation',
    description: 'Struktur & Einheiten',
    copy: {
      read: ['Organisation ansehen', 'Organisationsstruktur einsehen'],
      create: ['Organisation anlegen', 'Neue Organisation anlegen'],
      update: ['Organisation bearbeiten', 'Organisation umbenennen'],
      delete: ['Organisation löschen', 'Organisation löschen'],
    },
  },
  {
    resource: 'role',
    group: 'administration',
    label: 'Rollen & Rechte',
    description: 'Berechtigungen',
    copy: {
      read: ['Rollen ansehen', 'Rollen und ihre Rechte einsehen'],
      create: ['Rolle anlegen', 'Neue Rolle anlegen oder kopieren'],
      update: ['Rolle bearbeiten', 'Rechte einer Rolle ändern'],
      delete: ['Rolle löschen', 'Rolle löschen'],
    },
  },
]

const permissionFor = (resource: Resource, action: Action): Permission =>
  `${resource}:${action}`

export const PERMISSION_AREAS: PermissionArea[] = AREA_DEFINITIONS.map((definition) => ({
  resource: definition.resource,
  group: definition.group,
  label: definition.label,
  description: definition.description,
  actions: ACTIONS.map((action) => ({
    action,
    permission: permissionFor(definition.resource, action),
    label: definition.copy[action][0],
    hint: definition.copy[action][1],
  })),
}))

export const activeActionsOf = (resource: Resource, perms: ReadonlySet<string>): Action[] =>
  ACTIONS.filter((action) => perms.has(permissionFor(resource, action)))

export const activeActionCount = (resource: Resource, perms: ReadonlySet<string>): number =>
  activeActionsOf(resource, perms).length

export const levelOf = (
  resource: Resource,
  perms: ReadonlySet<string>,
): AccessLevel | 'custom' => {
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
