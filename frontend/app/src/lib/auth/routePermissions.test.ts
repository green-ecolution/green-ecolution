import { describe, it, expect } from 'vitest'
import { NO_PERMISSIONS, UNRESTRICTED, type Permissions } from './permissions'
import { ROUTE_PERMISSIONS, canAccessRoute, visibleNavSections } from './routePermissions'

const perms = (...granted: string[]): Permissions => new Set(granted)

describe('ROUTE_PERMISSIONS', () => {
  it('maps every gated section route', () => {
    expect(Object.keys(ROUTE_PERMISSIONS).sort()).toEqual([
      '/evaluations',
      '/map',
      '/sensors',
      '/team',
      '/treecluster',
      '/trees',
      '/vehicles',
      '/watering-plans',
    ])
  })
})

describe('canAccessRoute', () => {
  it('allows a mapped route when the permission is granted', () => {
    expect(canAccessRoute('/trees', perms('tree:read'))).toBe(true)
  })

  it('denies a mapped route when the permission is missing', () => {
    expect(canAccessRoute('/trees', perms('vehicle:read'))).toBe(false)
  })

  it('allows the map with either tree or cluster read', () => {
    expect(canAccessRoute('/map', perms('tree:read'))).toBe(true)
    expect(canAccessRoute('/map', perms('tree_cluster:read'))).toBe(true)
    expect(canAccessRoute('/map', perms('vehicle:update'))).toBe(false)
  })

  it('allows evaluations with any read grant', () => {
    expect(canAccessRoute('/evaluations', perms('vehicle:read'))).toBe(true)
    expect(canAccessRoute('/evaluations', perms('vehicle:update'))).toBe(false)
  })

  it('treats unmapped routes as open', () => {
    expect(canAccessRoute('/dashboard', NO_PERMISSIONS)).toBe(true)
    expect(canAccessRoute('/settings', NO_PERMISSIONS)).toBe(true)
    expect(canAccessRoute('/profile', NO_PERMISSIONS)).toBe(true)
    expect(canAccessRoute('/info', NO_PERMISSIONS)).toBe(true)
  })

  it('treats a missing or non-string target as open', () => {
    expect(canAccessRoute(undefined, NO_PERMISSIONS)).toBe(true)
  })

  it('allows everything for unrestricted access', () => {
    expect(canAccessRoute('/team', UNRESTRICTED)).toBe(true)
  })
})

describe('visibleNavSections', () => {
  const sections = [
    {
      id: 1,
      headline: 'Grünflächen',
      links: [
        { key: 'map', to: '/map' },
        { key: 'trees', to: '/trees' },
        { key: 'cluster', to: '/treecluster' },
      ],
    },
    {
      id: 2,
      headline: 'Einsatzplanung',
      links: [
        { key: 'plans', to: '/watering-plans' },
        { key: 'vehicles', to: '/vehicles' },
      ],
    },
  ]

  it('keeps only the links the user may reach', () => {
    const visible = visibleNavSections(sections, perms('tree:read'))

    expect(visible).toHaveLength(1)
    expect(visible[0].links.map((link) => link.key)).toEqual(['map', 'trees'])
  })

  it('drops sections that end up without links so no empty headline remains', () => {
    const visible = visibleNavSections(sections, perms('vehicle:read'))

    expect(visible.map((section) => section.headline)).toEqual(['Einsatzplanung'])
    expect(visible[0].links.map((link) => link.key)).toEqual(['vehicles'])
  })

  it('keeps everything for unrestricted access', () => {
    expect(visibleNavSections(sections, UNRESTRICTED)).toEqual(sections)
  })

  it('returns no section for a user without any grant', () => {
    expect(visibleNavSections(sections, NO_PERMISSIONS)).toEqual([])
  })

  it('does not mutate the input', () => {
    visibleNavSections(sections, NO_PERMISSIONS)

    expect(sections[0].links).toHaveLength(3)
  })
})
