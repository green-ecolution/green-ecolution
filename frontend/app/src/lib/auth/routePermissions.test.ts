import { describe, it, expect } from 'vitest'
import { z } from 'zod'
import {
  NO_PERMISSIONS,
  UNRESTRICTED,
  readPermissionsOf,
  resourcesFor,
  type Permissions,
} from './permissions'
import {
  CORE_ROUTE_PERMISSIONS,
  canAccessRoute as canAccessRouteIn,
  mapPermissionsFor,
  routePermissionsFor,
  visibleNavSections as visibleNavSectionsIn,
} from './routePermissions'
import { irrigationManifest } from '@green-ecolution/module-irrigation'
import type { ModuleManifest, ModuleMap } from '@/modules/types'
import { probeManifest } from '@/test/planningManifest'

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
    labelKey: 'irrigation:label',
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
    expect(table['/plans']).toBeUndefined()
  })

  it('takes the irrigation routes from its manifest', () => {
    expect(irrigationTable['/trees']).toEqual(['tree:read'])
    expect(irrigationTable['/treecluster']).toEqual(['tree_cluster:read'])
    expect(irrigationTable['/evaluations']).toEqual(
      readPermissionsOf(resourcesFor([irrigationManifest])),
    )
  })

  it('guards the plans with the plan resource of the active modules', () => {
    expect(irrigationTable['/plans']).toEqual(['watering_plan:read'])
  })

  it('opens the plans only for a module that can plan', () => {
    const operationsOnly: ModuleManifest = {
      ...probe,
      resources: [
        { key: 'vehicle', area: 'operations', textsKey: 'settings:permission.resource.vehicle' },
      ],
    }
    expect(routePermissionsFor([operationsOnly])['/plans']).toBeUndefined()
    expect(
      routePermissionsFor([probeManifest('waste', { planResource: 'sensor' })])['/plans'],
    ).toEqual(['sensor:read'])
  })

  it('does not apply a route requirement of an inactive module', () => {
    expect(canAccessRouteIn('/probe', perms('vehicle:read'), routePermissionsFor([]))).toBe(true)
    expect(canAccessRouteIn('/probe', perms('vehicle:read'), routePermissionsFor([probe]))).toBe(
      false,
    )
  })
})

describe('mapPermissionsFor', () => {
  const withMap = (key: string, permissions: ModuleMap['permissions']): ModuleManifest => ({
    key,
    labelKey: 'irrigation:label',
    i18n: { namespaces: [], load: () => Promise.resolve({}) },
    map: {
      permissions,
      searchSchema: z.object({}),
      interactiveLayerIds: [],
      Overview: () => null,
    },
  })

  it('unites the map requirements of the given manifests without duplicates', () => {
    expect(
      mapPermissionsFor([
        withMap('alpha', ['sensor:read', 'vehicle:read']),
        withMap('beta', ['vehicle:read', 'region:read']),
      ]),
    ).toEqual(['sensor:read', 'vehicle:read', 'region:read'])
  })

  it('asks the same as before for the map with irrigation active', () => {
    expect(irrigationTable['/map']).toEqual(['tree:read', 'tree_cluster:read'])
  })

  it('takes the map requirement from the active manifests only', () => {
    const table = routePermissionsFor([withMap('alpha', ['sensor:read'])])
    expect(table['/map']).toEqual(['sensor:read'])
  })

  it('leaves the map open when no active manifest contributes to it', () => {
    expect(routePermissionsFor([])['/map']).toBeUndefined()
    expect(canAccessRouteIn('/map', NO_PERMISSIONS, routePermissionsFor([]))).toBe(true)
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
        { key: 'plans', to: '/plans' },
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
