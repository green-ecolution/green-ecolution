import { describe, it, expect } from 'vitest'
import { ANY_READ, NO_PERMISSIONS, UNRESTRICTED, type Permissions } from './permissions'
import {
  CORE_ROUTE_PERMISSIONS,
  canAccessRoute as canAccessRouteIn,
  routePermissionsFor,
  visibleNavSections as visibleNavSectionsIn,
} from './routePermissions'
import { irrigationManifest } from '@green-ecolution/module-irrigation'
import type { ModuleManifest } from '@/modules/types'

const perms = (...granted: string[]): Permissions => new Set(granted)

const irrigationTable = routePermissionsFor([irrigationManifest])
const canAccessRoute = (path: unknown, granted: Permissions) =>
  canAccessRouteIn(path, granted, irrigationTable)
const visibleNavSections = <S extends { links: readonly { to?: unknown }[] }>(
  sections: readonly S[],
  granted: Permissions,
) => visibleNavSectionsIn(sections, granted, irrigationTable)

describe('CORE_ROUTE_PERMISSIONS', () => {
  it('carries no irrigation route', () => {
    expect(Object.keys(CORE_ROUTE_PERMISSIONS).sort()).toEqual([
      '/map',
      '/sensors',
      '/settings/team/members',
      '/settings/team/roles',
      '/vehicles',
    ])
  })
})

describe('routePermissionsFor', () => {
  const probe: ModuleManifest = {
    key: 'probe',
    labelKey: 'common:modules.irrigation',
    i18n: { namespaces: [], load: () => Promise.resolve({}) },
    routePermissions: { '/probe': ['sensor:read'] },
  }

  it('adds the routes of the given manifests to the core table', () => {
    const table = routePermissionsFor([probe])
    expect(table['/probe']).toEqual(['sensor:read'])
    expect(table['/vehicles']).toEqual(['vehicle:read'])
  })

  it('leaves module routes out when no manifest is active', () => {
    const table = routePermissionsFor([])
    expect(Object.keys(table).sort()).toEqual(Object.keys(CORE_ROUTE_PERMISSIONS).sort())
    expect(table['/trees']).toBeUndefined()
    expect(table['/watering-plans']).toBeUndefined()
  })

  it('takes the irrigation routes from its manifest', () => {
    expect(irrigationTable['/trees']).toEqual(['tree:read'])
    expect(irrigationTable['/treecluster']).toEqual(['tree_cluster:read'])
    expect(irrigationTable['/evaluations']).toEqual(ANY_READ)
  })

  it('guards the plans with the plan resource of the active modules', () => {
    expect(irrigationTable['/watering-plans']).toEqual(['watering_plan:read'])
  })

  it('does not apply a route requirement of an inactive module', () => {
    expect(canAccessRouteIn('/probe', perms('vehicle:read'), routePermissionsFor([]))).toBe(true)
    expect(canAccessRouteIn('/probe', perms('vehicle:read'), routePermissionsFor([probe]))).toBe(
      false,
    )
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
    expect(canAccessRoute('/settings/team/members', UNRESTRICTED)).toBe(true)
  })

  it('guards the roles tab with role:read', () => {
    expect(canAccessRoute('/settings/team/roles', new Set(['user:read']))).toBe(false)
    expect(canAccessRoute('/settings/team/roles', new Set(['role:read']))).toBe(true)
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
      headline: 'Planung',
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

    expect(visible.map((section) => section.headline)).toEqual(['Planung'])
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
