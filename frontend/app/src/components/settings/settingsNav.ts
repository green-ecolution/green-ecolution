import type { LinkProps } from '@tanstack/react-router'
import type de from '@/locales/de'
import { satisfies, type PermissionRequirement, type Permissions } from '@/lib/auth/permissions'
import { SECTION_ROUTES } from './settingsSections'

type SettingsNavKey = keyof (typeof de)['settings']['nav']

export type SettingsNavGroup = 'account' | 'organization'

export interface SettingsNavItem {
  key: SettingsNavKey
  group: SettingsNavGroup
  /** Lucide icon name, resolved in SettingsLayout so this module stays JSX-free. */
  icon: string
  /** Organization entries carry `$orgId`, filled in by the layout. */
  to: LinkProps['to']
  permission?: PermissionRequirement
  featureKey?: string
  comingSoon?: boolean
  /** Pages about one tenant's operations; the instance admin has none. */
  hiddenInAdminMode?: boolean
  /** Shows the number of sub-organizations or members next to the label. */
  count?: 'children' | 'members'
}

export const SETTINGS_NAV: SettingsNavItem[] = [
  { key: 'profile', group: 'account', icon: 'UserRound', to: '/settings/profile' },
  {
    key: 'notifications',
    group: 'account',
    icon: 'Bell',
    to: '/settings/notifications',
    comingSoon: true,
  },
  {
    key: 'overview',
    group: 'organization',
    icon: 'Building2',
    to: SECTION_ROUTES.overview,
    permission: ['organization:read'],
  },
  {
    key: 'areas',
    group: 'organization',
    icon: 'SlidersHorizontal',
    to: SECTION_ROUTES.areas,
    permission: ['setting:read'],
  },
  {
    key: 'structure',
    group: 'organization',
    icon: 'Network',
    to: SECTION_ROUTES.structure,
    permission: ['organization:read'],
    count: 'children',
  },
  {
    key: 'members',
    group: 'organization',
    icon: 'Users',
    to: SECTION_ROUTES.members,
    permission: ['user:read'],
    count: 'members',
  },
  {
    key: 'roles',
    group: 'organization',
    icon: 'ShieldCheck',
    to: SECTION_ROUTES.roles,
    permission: ['role:read'],
  },
  {
    key: 'stations',
    group: 'organization',
    icon: 'MapPin',
    to: SECTION_ROUTES.stations,
    featureKey: 'routing',
    permission: ['start_point:read'],
    hiddenInAdminMode: true,
  },
  {
    key: 'plugins',
    group: 'organization',
    icon: 'Puzzle',
    to: SECTION_ROUTES.plugins,
    featureKey: 'plugins',
    permission: ['plugin:read'],
  },
]

/** `perms` are the rights in the selected organization, not the union over all. */
export const visibleSettingsNav = (
  items: SettingsNavItem[],
  perms: Permissions,
  enabledFeatures: ReadonlySet<string>,
  adminMode = false,
): SettingsNavItem[] =>
  items.filter((item) => {
    if (adminMode && item.hiddenInAdminMode) return false
    if (item.featureKey && !enabledFeatures.has(item.featureKey)) return false
    if (item.permission && !satisfies(perms, item.permission)) return false
    return true
  })
