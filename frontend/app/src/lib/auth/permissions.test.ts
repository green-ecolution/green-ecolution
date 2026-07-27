import { describe, it, expect } from 'vitest'
import type { UserResponse } from '@green-ecolution/backend-client'
import {
  ANY_READ,
  NO_PERMISSIONS,
  UNRESTRICTED,
  permissionsOf,
  satisfies,
  RESOURCES,
} from './permissions'

// Fixture: permissionsOf only reads `roles`, so the rest of UserResponse is irrelevant.
const userWithRoles = (...permissionSets: string[][]): UserResponse =>
  ({
    roles: permissionSets.map((permissions, index) => ({
      id: `role-${index}`,
      name: `Rolle ${index}`,
      description: '',
      organizationId: null,
      permissions,
      createdAt: '2026-07-27T00:00:00Z',
    })),
  }) as unknown as UserResponse

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

  it('ANY_READ covers every resource and matches any single read grant', () => {
    expect(ANY_READ).toHaveLength(RESOURCES.length)
    expect(satisfies(permissionsOf(userWithRoles(['vehicle:read'])), ANY_READ)).toBe(true)
    expect(satisfies(permissionsOf(userWithRoles(['vehicle:update'])), ANY_READ)).toBe(false)
  })
})
