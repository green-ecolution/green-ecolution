import { describe, it, expect } from 'vitest'
import type { OrganizationResponse, RoleResponse } from '@green-ecolution/backend-client'
import { orgsWithPermission } from './organizationScope'

// Root ─ TBZ ─ Extern, plus a sibling tenant that shares no ancestry with TBZ.
const orgs = [
  { id: 'root', name: 'Green Ecolution', parentId: null, memberCount: 0 },
  { id: 'tbz', name: 'TBZ', parentId: 'root', memberCount: 0 },
  { id: 'extern', name: 'Extern A', parentId: 'tbz', memberCount: 0 },
  { id: 'other', name: 'Andere Stadt', parentId: 'root', memberCount: 0 },
] as unknown as OrganizationResponse[]

const role = (organizationId: string | null, permissions: string[]): RoleResponse =>
  ({ id: `role-${organizationId}`, name: 'Rolle', organizationId, permissions }) as RoleResponse

describe('orgsWithPermission', () => {
  it('includes the role organization and its whole subtree', () => {
    const result = orgsWithPermission(
      orgs,
      [role('tbz', ['tree_cluster:create'])],
      'tree_cluster:create',
    )

    expect(result.map((o) => o.id)).toEqual(['extern', 'tbz'])
  })

  it('ignores roles that lack the permission', () => {
    const result = orgsWithPermission(
      orgs,
      [role('tbz', ['tree_cluster:read'])],
      'tree_cluster:create',
    )

    expect(result).toEqual([])
  })

  it('ignores template roles without an organization', () => {
    const result = orgsWithPermission(
      orgs,
      [role(null, ['tree_cluster:create'])],
      'tree_cluster:create',
    )

    expect(result).toEqual([])
  })

  it('never grants upwards from the role organization', () => {
    const result = orgsWithPermission(
      orgs,
      [role('extern', ['tree_cluster:create'])],
      'tree_cluster:create',
    )

    expect(result.map((o) => o.id)).toEqual(['extern'])
  })

  it('unions overlapping subtrees without duplicates', () => {
    const result = orgsWithPermission(
      orgs,
      [role('tbz', ['tree_cluster:create']), role('extern', ['tree_cluster:create'])],
      'tree_cluster:create',
    )

    expect(result.map((o) => o.id)).toEqual(['extern', 'tbz'])
  })

  it('drops organizations the caller cannot see at all', () => {
    // The org list is already scoped by the backend; a grant for an absent org
    // must not conjure it back into the picker.
    const visible = orgs.filter((o) => o.id !== 'other')
    const result = orgsWithPermission(
      visible,
      [role('other', ['tree_cluster:create'])],
      'tree_cluster:create',
    )

    expect(result).toEqual([])
  })
})
