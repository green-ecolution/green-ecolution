import type { LinkProps } from '@tanstack/react-router'

export type SettingsSection =
  'overview' | 'areas' | 'structure' | 'members' | 'roles' | 'stations' | 'plugins'

export const SECTION_ROUTES = {
  overview: '/settings/organization/$orgId',
  areas: '/settings/organization/$orgId/areas',
  structure: '/settings/organization/$orgId/structure',
  members: '/settings/organization/$orgId/members',
  roles: '/settings/organization/$orgId/roles',
  stations: '/settings/organization/$orgId/stations',
  plugins: '/settings/organization/$orgId/plugins',
} as const satisfies Record<SettingsSection, LinkProps['to']>

const ORGANIZATION_PATH = /^\/settings\/organization\/([^/]+)(?:\/([^/]+))?/

export function organizationIdOf(pathname: string): string | null {
  return ORGANIZATION_PATH.exec(pathname)?.[1] ?? null
}

/** The sub-page a path belongs to; a detail below it counts as its list. */
export function sectionOf(pathname: string): SettingsSection | null {
  const match = ORGANIZATION_PATH.exec(pathname)
  if (!match) return null
  const segment = match[2]
  if (!segment) return 'overview'
  return segment in SECTION_ROUTES ? (segment as SettingsSection) : null
}
