import { describe, it, expect } from 'vitest'
import type { Role } from '@/api/backendApi'
import { isPristineTemplateCopy, ownRolesOf, roleDisplayName, samePermissionSet } from './roleList'

const role = (overrides: Partial<Role>): Role => ({
  id: 'id',
  name: 'Name',
  isTemplate: false,
  permissions: [],
  ...overrides,
})

const templates: Role[] = [
  role({
    id: 't1',
    name: 'Baumpflege',
    isTemplate: true,
    templateKey: 'tree_care',
    permissions: ['tree:read', 'tree:create'],
  }),
  role({
    id: 't2',
    name: 'Beobachter',
    isTemplate: true,
    templateKey: 'observer',
    permissions: ['tree:read'],
  }),
]

describe('samePermissionSet', () => {
  it('ignores order and duplicates', () => {
    expect(samePermissionSet(['b', 'a'], ['a', 'b'])).toBe(true)
    expect(samePermissionSet(['a', 'a', 'b'], ['a', 'b'])).toBe(true)
  })

  it('detects a difference', () => {
    expect(samePermissionSet(['a'], ['a', 'b'])).toBe(false)
  })
})

describe('isPristineTemplateCopy', () => {
  it('matches a copy the backend still marks as delivered', () => {
    const copy = role({ id: 'c1', name: 'Baumpflege', templateKey: 'tree_care' })
    expect(isPristineTemplateCopy(copy)).toBe(true)
  })

  it('does not match once the backend cleared the key on an edit', () => {
    const copy = role({ id: 'c1', name: 'Baumpflege Nord', templateKey: null })
    expect(isPristineTemplateCopy(copy)).toBe(false)
  })

  it('does not match a role that never came from a template', () => {
    const own = role({ id: 'c2', name: 'Praktikant' })
    expect(isPristineTemplateCopy(own)).toBe(false)
  })

  it('does not match a template itself', () => {
    expect(isPristineTemplateCopy(templates[0])).toBe(false)
  })
})

describe('ownRolesOf', () => {
  it('drops the untouched template copies and keeps the rest', () => {
    const roles: Role[] = [
      role({ id: 'c1', name: 'Baumpflege', templateKey: 'tree_care' }),
      role({ id: 'c2', name: 'Beobachter Nord', templateKey: null }),
      role({ id: 'c3', name: 'Praktikant' }),
    ]
    expect(ownRolesOf(roles).map((entry) => entry.id)).toEqual(['c2', 'c3'])
  })

  it('never lists templates themselves', () => {
    expect(ownRolesOf(templates)).toEqual([])
  })
})

describe('roleDisplayName', () => {
  it('translates a delivered role through its template key', () => {
    expect(roleDisplayName(role({ name: 'Baumpflege', templateKey: 'tree_care' }))).toBe(
      'Baumpflege',
    )
  })

  it('shows the stored name once the role has been edited', () => {
    expect(roleDisplayName(role({ name: 'Pflege Nord', templateKey: undefined }))).toBe(
      'Pflege Nord',
    )
  })

  it('falls back to the stored name for an unknown template key', () => {
    expect(roleDisplayName(role({ name: 'Sonderrolle', templateKey: 'not_in_catalog' }))).toBe(
      'Sonderrolle',
    )
  })
})
