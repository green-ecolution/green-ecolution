import { satisfies, type PermissionRequirement, type Permissions } from '@/lib/auth/permissions'

export interface SettingsNavItem {
  key: string
  label: string
  /** Lucide icon name, resolved in SettingsLayout so this module stays JSX-free. */
  icon: string
  to: string
  permission?: PermissionRequirement
  featureKey?: string
  comingSoon?: boolean
}

export const SETTINGS_NAV: SettingsNavItem[] = [
  { key: 'profile', label: 'Profil', icon: 'UserRound', to: '/settings/profile' },
  {
    key: 'organization',
    label: 'Organisation',
    icon: 'Building2',
    to: '/settings/organization',
    permission: ['organization:read'],
  },
  {
    key: 'irrigation',
    label: 'Bewässerung',
    icon: 'Droplet',
    to: '/settings/irrigation',
    comingSoon: true,
  },
  {
    key: 'notifications',
    label: 'Benachrichtigungen',
    icon: 'Bell',
    to: '/settings/notifications',
    comingSoon: true,
  },
  {
    key: 'sensors',
    label: 'Sensoren & Netz',
    icon: 'RadioTower',
    to: '/settings/sensors',
    comingSoon: true,
  },
  {
    key: 'team',
    label: 'Team & Rollen',
    icon: 'Users',
    to: '/settings/team',
    permission: ['user:read', 'role:read'],
  },
  { key: 'map', label: 'Karte & Einheiten', icon: 'Map', to: '/settings/map', comingSoon: true },
  {
    key: 'plugin',
    label: 'Plugins',
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
