import { describe, it, expect } from 'vitest'
import type { Role } from '@/api/backendApi'
import { isPristineTemplateCopy, ownRolesOf, samePermissionSet } from './roleList'

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
    permissions: ['tree:read', 'tree:create'],
  }),
  role({ id: 't2', name: 'Beobachter', isTemplate: true, permissions: ['tree:read'] }),
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
  it('matches a copy with the same name and the same permissions', () => {
    const copy = role({ id: 'c1', name: 'Baumpflege', permissions: ['tree:create', 'tree:read'] })
    expect(isPristineTemplateCopy(copy, templates)).toBe(true)
  })

  it('does not match once the permissions changed', () => {
    const copy = role({ id: 'c1', name: 'Baumpflege', permissions: ['tree:read'] })
    expect(isPristineTemplateCopy(copy, templates)).toBe(false)
  })

  it('does not match once the name changed', () => {
    const copy = role({
      id: 'c1',
      name: 'Baumpflege Nord',
      permissions: ['tree:read', 'tree:create'],
    })
    expect(isPristineTemplateCopy(copy, templates)).toBe(false)
  })

  it('does not match a role unrelated to any template', () => {
    const own = role({ id: 'c2', name: 'Praktikant', permissions: ['tree:read', 'sensor:read'] })
    expect(isPristineTemplateCopy(own, templates)).toBe(false)
  })
})

describe('ownRolesOf', () => {
  it('drops the untouched template copies and keeps the rest', () => {
    const roles: Role[] = [
      role({ id: 'c1', name: 'Baumpflege', permissions: ['tree:read', 'tree:create'] }),
      role({ id: 'c2', name: 'Beobachter', permissions: ['tree:read', 'sensor:read'] }),
      role({ id: 'c3', name: 'Praktikant', permissions: ['tree:read'] }),
    ]
    expect(ownRolesOf(roles, templates).map((entry) => entry.id)).toEqual(['c2', 'c3'])
  })

  it('never lists templates themselves', () => {
    expect(ownRolesOf(templates, templates)).toEqual([])
  })
})
