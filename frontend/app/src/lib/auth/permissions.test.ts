import { describe, it, expect } from 'vitest'
import type { RoleResponse } from '@green-ecolution/backend-client'
import {
  NO_PERMISSIONS,
  UNRESTRICTED,
  permissionsOf,
  satisfies,
  CORE_RESOURCES,
  readPermissionsOf,
  resourcesFor,
} from './permissions'
import { irrigationManifest } from '@green-ecolution/module-irrigation'

const userWithRoles = (...permissionSets: string[][]): RoleResponse[] =>
  permissionSets.map(
    (permissions, index) =>
      ({
        id: `role-${index}`,
        name: `Rolle ${index}`,
        description: '',
        organizationId: null,
        permissions,
        createdAt: '2026-07-27T00:00:00Z',
      }) as unknown as RoleResponse,
  )

describe('permissionsOf', () => {
  it('unions the permissions of all assigned roles', () => {
    const perms = permissionsOf(userWithRoles(['tree:read'], ['vehicle:read', 'vehicle:update']))

    expect(perms).not.toBe(UNRESTRICTED)
    expect([...(perms as ReadonlySet<string>)].sort()).toEqual([
      'tree:read',
      'vehicle:read',
      'vehicle:update',
    ])
  })

  it('deduplicates permissions granted by several roles', () => {
    const perms = permissionsOf(userWithRoles(['tree:read'], ['tree:read']))

    expect([...(perms as ReadonlySet<string>)]).toEqual(['tree:read'])
  })

  it('returns an empty set for a user without roles', () => {
    expect([...(permissionsOf(userWithRoles()) as ReadonlySet<string>)]).toEqual([])
  })

  it('returns an empty set when the user is not loaded yet', () => {
    expect([...(permissionsOf(undefined) as ReadonlySet<string>)]).toEqual([])
  })

  it('keeps permission strings the frontend does not know', () => {
    const perms = permissionsOf(userWithRoles(['future_resource:read']))

    expect((perms as ReadonlySet<string>).has('future_resource:read')).toBe(true)
  })
})

describe('satisfies', () => {
  it('accepts when the required permission is present', () => {
    expect(satisfies(permissionsOf(userWithRoles(['tree:read'])), ['tree:read'])).toBe(true)
  })

  it('rejects when the required permission is missing', () => {
    expect(satisfies(permissionsOf(userWithRoles(['tree:read'])), ['vehicle:read'])).toBe(false)
  })

  it('treats a requirement as OR: one of several is enough', () => {
    const perms = permissionsOf(userWithRoles(['tree_cluster:read']))

    expect(satisfies(perms, ['tree:read', 'tree_cluster:read'])).toBe(true)
  })

  it('always accepts unrestricted access', () => {
    expect(satisfies(UNRESTRICTED, ['organization:delete'])).toBe(true)
  })

  it('always accepts an empty requirement', () => {
    expect(satisfies(NO_PERMISSIONS, [])).toBe(true)
  })

  it('a read requirement over several resources matches any single read grant', () => {
    const anyRead = readPermissionsOf(resourcesFor([irrigationManifest]))
    expect(anyRead).toHaveLength(resourcesFor([irrigationManifest]).length)
    expect(satisfies(permissionsOf(userWithRoles(['vehicle:read'])), anyRead)).toBe(true)
    expect(satisfies(permissionsOf(userWithRoles(['vehicle:update'])), anyRead)).toBe(false)
  })
})

describe('resourcesFor', () => {
  it('keeps the module resources out of the core list', () => {
    expect(CORE_RESOURCES).not.toContain('tree')
    expect(CORE_RESOURCES).not.toContain('tree_cluster')
    expect(CORE_RESOURCES).not.toContain('watering_plan')
    expect(resourcesFor([])).toEqual([...CORE_RESOURCES])
  })

  it('adds the resources of the given manifests', () => {
    expect([...resourcesFor([irrigationManifest])].sort()).toEqual(
      [...CORE_RESOURCES, 'tree', 'tree_cluster', 'watering_plan'].sort(),
    )
  })
})
