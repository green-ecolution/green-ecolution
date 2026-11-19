import { describe, it, expect } from 'vitest'
import { UNRESTRICTED } from '@/lib/auth/permissions'
import { landingSection, sectionPermission, SETTINGS_NAV, visibleSettingsNav } from './settingsNav'

const keysOf = (items: { key: string }[]) => items.map((item) => item.key)
const ALL_FEATURES = new Set(['plugins', 'routing'])

describe('SETTINGS_NAV', () => {
  it('lists the account pages, then the pages of the organization', () => {
    expect(SETTINGS_NAV.map((item) => [item.group, item.key])).toEqual([
      ['account', 'profile'],
      ['account', 'notifications'],
      ['organization', 'overview'],
      ['organization', 'areas'],
      ['organization', 'structure'],
      ['organization', 'members'],
      ['organization', 'roles'],
      ['organization', 'stations'],
      ['organization', 'plugins'],
    ])
  })

  it('marks only notifications as coming soon', () => {
    expect(keysOf(SETTINGS_NAV.filter((item) => item.comingSoon))).toEqual(['notifications'])
  })
})

describe('visibleSettingsNav', () => {
  it('always keeps the account pages', () => {
    const visible = visibleSettingsNav(SETTINGS_NAV, new Set(), new Set())
    expect(keysOf(visible)).toEqual(['profile', 'notifications'])
  })

  it('shows every page with every right and feature', () => {
    const visible = visibleSettingsNav(SETTINGS_NAV, UNRESTRICTED, ALL_FEATURES)
    expect(keysOf(visible)).toHaveLength(SETTINGS_NAV.length)
  })

  it('shows overview and structure with organization:read alone', () => {
    const visible = visibleSettingsNav(SETTINGS_NAV, new Set(['organization:read']), ALL_FEATURES)
    expect(keysOf(visible)).toEqual(['profile', 'notifications', 'overview', 'structure'])
  })

  it('ties areas to setting:read, members to user:read and roles to role:read', () => {
    const perms = new Set(['setting:read', 'user:read', 'role:read'])
    const visible = visibleSettingsNav(SETTINGS_NAV, perms, new Set())
    expect(keysOf(visible)).toEqual(['profile', 'notifications', 'areas', 'members', 'roles'])
  })

  it('hides plugins and stations when their feature is off', () => {
    const visible = visibleSettingsNav(SETTINGS_NAV, UNRESTRICTED, new Set())
    expect(keysOf(visible)).not.toContain('plugins')
    expect(keysOf(visible)).not.toContain('stations')
  })

  it('hides stations at the instance root', () => {
    const visible = visibleSettingsNav(SETTINGS_NAV, UNRESTRICTED, ALL_FEATURES, true)
    expect(keysOf(visible)).not.toContain('stations')
    expect(keysOf(visible)).toContain('plugins')
  })
})

describe('sectionPermission', () => {
  it("asks for the right of the section's navigation entry", () => {
    expect(sectionPermission('overview')).toEqual(['organization:read'])
    expect(sectionPermission('stations')).toEqual(['start_point:read'])
    expect(sectionPermission(null)).toBeUndefined()
  })
})

describe('landingSection', () => {
  it('opens the overview with organization:read', () => {
    expect(landingSection(new Set(['organization:read']), ALL_FEATURES, false)).toBe('overview')
  })

  it('opens the first section the rights allow, in navigation order', () => {
    expect(landingSection(new Set(['role:read', 'user:read']), ALL_FEATURES, false)).toBe('members')
    expect(landingSection(new Set(['start_point:read']), ALL_FEATURES, false)).toBe('stations')
  })

  it('applies the feature flags and the instance-root rule', () => {
    const perms = new Set(['start_point:read'])
    expect(landingSection(perms, new Set(), false)).toBeNull()
    expect(landingSection(perms, ALL_FEATURES, true)).toBeNull()
  })

  it('keeps a preferred section that is visible and replaces one that is not', () => {
    const perms = new Set(['role:read', 'user:read'])
    expect(landingSection(perms, ALL_FEATURES, false, 'roles')).toBe('roles')
    expect(landingSection(perms, ALL_FEATURES, false, 'plugins')).toBe('members')
  })

  it('finds nothing without any right', () => {
    expect(landingSection(new Set(), ALL_FEATURES, false)).toBeNull()
  })
})
