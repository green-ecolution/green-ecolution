import { describe, it, expect } from 'vitest'
import { UNRESTRICTED } from '@/lib/auth/permissions'
import { SETTINGS_NAV, visibleSettingsNav } from './settingsNav'

const keysOf = (items: { key: string }[]) => items.map((item) => item.key)

describe('SETTINGS_NAV', () => {
  it('lists the six settings pages in order', () => {
    expect(keysOf(SETTINGS_NAV)).toEqual([
      'profile',
      'myOrganization',
      'organization',
      'notifications',
      'team',
      'plugin',
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

  it('offers the own organization shortcut only with both setting:read and organization:read', () => {
    expect(
      keysOf(
        visibleSettingsNav(SETTINGS_NAV, new Set(['setting:read', 'organization:read']), new Set()),
      ),
    ).toContain('myOrganization')
    expect(
      keysOf(visibleSettingsNav(SETTINGS_NAV, new Set(['organization:read']), new Set())),
    ).not.toContain('myOrganization')
    // The Beobachter template holds exactly this combination: setting:read without
    // organization:read. The page it links to fetches the organization tree
    // regardless, so setting:read alone would land on a load error.
    expect(
      keysOf(visibleSettingsNav(SETTINGS_NAV, new Set(['setting:read']), new Set())),
    ).not.toContain('myOrganization')
  })
})
