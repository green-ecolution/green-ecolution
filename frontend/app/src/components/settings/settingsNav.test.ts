import { describe, it, expect } from 'vitest'
import { UNRESTRICTED } from '@/lib/auth/permissions'
import { sectionPermission, SETTINGS_NAV, visibleSettingsNav } from './settingsNav'

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
