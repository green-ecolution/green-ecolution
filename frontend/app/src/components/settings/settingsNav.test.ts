import { describe, it, expect } from 'vitest'
import { UNRESTRICTED } from '@/lib/auth/permissions'
import { SETTINGS_NAV, visibleSettingsNav } from './settingsNav'

const keysOf = (items: { key: string }[]) => items.map((item) => item.key)

describe('SETTINGS_NAV', () => {
  it('lists the settings pages in order', () => {
    expect(keysOf(SETTINGS_NAV)).toEqual([
      'profile',
      'organization',
      'notifications',
      'team',
      'plugin',
      'stations',
    ])
  })

  it('marks only notifications as coming soon', () => {
    const comingSoon = SETTINGS_NAV.filter((item) => item.comingSoon).map((item) => item.key)
    expect(comingSoon).toEqual(['notifications'])
  })

  it('no longer offers the placeholder areas whose values moved into the organization', () => {
    const keys = keysOf(SETTINGS_NAV)
    expect(keys).not.toContain('irrigation')
    expect(keys).not.toContain('sensors')
    expect(keys).not.toContain('map')
    expect(keys).toContain('notifications')
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

  it('shows stations only with start point read and routing enabled', () => {
    const perms = new Set(['start_point:read'])
    expect(keysOf(visibleSettingsNav(SETTINGS_NAV, perms, new Set(['routing'])))).toContain(
      'stations',
    )
    expect(keysOf(visibleSettingsNav(SETTINGS_NAV, perms, new Set()))).not.toContain('stations')
    expect(
      keysOf(visibleSettingsNav(SETTINGS_NAV, new Set(['tree:read']), new Set(['routing']))),
    ).not.toContain('stations')
  })
})

describe('visibleSettingsNav in admin mode', () => {
  it('hides the stations entry, which belongs to a tenant', () => {
    const features = new Set(['routing', 'plugins'])
    const normal = visibleSettingsNav(SETTINGS_NAV, UNRESTRICTED, features)
    const admin = visibleSettingsNav(SETTINGS_NAV, UNRESTRICTED, features, true)
    expect(keysOf(normal)).toContain('stations')
    expect(keysOf(admin)).not.toContain('stations')
    expect(keysOf(admin)).toContain('organization')
  })
})
