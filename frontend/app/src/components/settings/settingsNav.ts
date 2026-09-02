import type de from '@/locales/de'
import { satisfies, type PermissionRequirement, type Permissions } from '@/lib/auth/permissions'

type SettingsNavKey = keyof (typeof de)['settings']['nav']

export interface SettingsNavItem {
  key: string
  /** Key into settings:nav.*, resolved in SettingsLayout so this module stays translation-free. */
  labelKey: SettingsNavKey
  /** Lucide icon name, resolved in SettingsLayout so this module stays JSX-free. */
  icon: string
  to: string
  permission?: PermissionRequirement
  featureKey?: string
  comingSoon?: boolean
}

export const SETTINGS_NAV: SettingsNavItem[] = [
  { key: 'profile', labelKey: 'profile', icon: 'UserRound', to: '/settings/profile' },
  {
    key: 'organization',
    labelKey: 'organization',
    icon: 'Building2',
    to: '/settings/organization',
    permission: ['organization:read'],
  },
  {
    key: 'irrigation',
    labelKey: 'irrigation',
    icon: 'Droplet',
    to: '/settings/irrigation',
    comingSoon: true,
  },
  {
    key: 'notifications',
    labelKey: 'notifications',
    icon: 'Bell',
    to: '/settings/notifications',
    comingSoon: true,
  },
  {
    key: 'sensors',
    labelKey: 'sensors',
    icon: 'RadioTower',
    to: '/settings/sensors',
    comingSoon: true,
  },
  {
    key: 'team',
    labelKey: 'team',
    icon: 'Users',
    to: '/settings/team',
    permission: ['user:read', 'role:read'],
  },
  {
    key: 'map',
    labelKey: 'map',
    icon: 'Map',
    to: '/settings/map',
    comingSoon: true,
  },
  {
    key: 'plugin',
    labelKey: 'plugin',
    icon: 'Puzzle',
    to: '/settings/plugin',
    featureKey: 'plugins',
  },
]

export const visibleSettingsNav = (
  items: SettingsNavItem[],
  perms: Permissions,
  enabledFeatures: ReadonlySet<string>,
): SettingsNavItem[] =>
  items.filter((item) => {
    if (item.featureKey && !enabledFeatures.has(item.featureKey)) return false
    if (item.permission && !satisfies(perms, item.permission)) return false
    return true
  })
