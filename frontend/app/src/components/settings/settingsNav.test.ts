import { describe, it, expect } from 'vitest'
import { UNRESTRICTED } from '@/lib/auth/permissions'
import { SETTINGS_NAV, visibleSettingsNav } from './settingsNav'

const keysOf = (items: { key: string }[]) => items.map((item) => item.key)

describe('SETTINGS_NAV', () => {
  it('lists the eight settings pages in order', () => {
    expect(keysOf(SETTINGS_NAV)).toEqual([
      'profile',
      'organization',
      'irrigation',
      'notifications',
      'sensors',
      'team',
      'map',
      'plugin',
    ])
  })

  it('marks the four unbuilt pages as coming soon', () => {
    const comingSoon = SETTINGS_NAV.filter((item) => item.comingSoon).map((item) => item.key)
    expect(comingSoon).toEqual(['irrigation', 'notifications', 'sensors', 'map'])
  })
})

describe('visibleSettingsNav', () => {
  it('hides the team entry without user or role read', () => {
    const visible = visibleSettingsNav(SETTINGS_NAV, new Set(['tree:read']), new Set(['plugins']))
    expect(keysOf(visible)).not.toContain('team')
  })

  it('shows the team entry with role read alone', () => {
    const visible = visibleSettingsNav(SETTINGS_NAV, new Set(['role:read']), new Set())
    expect(keysOf(visible)).toContain('team')
  })

  it('hides the organization entry without organization read', () => {
    const visible = visibleSettingsNav(SETTINGS_NAV, new Set(['tree:read']), new Set())
    expect(keysOf(visible)).not.toContain('organization')
  })

  it('hides plugins when the feature is disabled', () => {
    const visible = visibleSettingsNav(SETTINGS_NAV, UNRESTRICTED, new Set())
    expect(keysOf(visible)).not.toContain('plugin')
  })

  it('shows plugins when the feature is enabled', () => {
    const visible = visibleSettingsNav(SETTINGS_NAV, UNRESTRICTED, new Set(['plugins']))
    expect(keysOf(visible)).toContain('plugin')
  })

  it('always keeps profile', () => {
    const visible = visibleSettingsNav(SETTINGS_NAV, new Set(), new Set())
    expect(keysOf(visible)).toContain('profile')
  })
})
