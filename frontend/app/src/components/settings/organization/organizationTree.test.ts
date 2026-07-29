import { describe, it, expect } from 'vitest'
import type { OrganizationResponse } from '@/api/backendApi'
import { buildTree, flatten, pathTo, subtreeMemberCount } from './organizationTree'

const org = (
  id: string,
  parentId: string | null,
  name: string,
  memberCount = 0,
): OrganizationResponse => ({
  id,
  parentId,
  name,
  address: null,
  contactPersonId: null,
  memberCount,
  createdAt: null,
})

const orgs = [
  org('root', null, 'Green Ecolution'),
  org('amt', 'root', 'Grünflächenamt', 2),
  org('nord', 'amt', 'Stadtgärtnerei Nord', 4),
  org('duburg', 'nord', 'Team Duburg', 6),
  org('juergensby', 'nord', 'Team Jürgensby', 5),
  org('orphan', 'gone', 'Verwaiste Einheit', 9),
]

describe('buildTree', () => {
  it('roots the tree at the given organization', () => {
    const tree = buildTree(orgs, 'amt')
    expect(tree?.org.id).toBe('amt')
    expect(tree?.children.map((c) => c.org.id)).toEqual(['nord'])
  })

  it('nests grandchildren', () => {
    const tree = buildTree(orgs, 'amt')
    expect(tree?.children[0].children.map((c) => c.org.id)).toEqual(['duburg', 'juergensby'])
  })

  it('sorts siblings by name', () => {
    const reversed = [...orgs].reverse()
    const tree = buildTree(reversed, 'nord')
    expect(tree?.children.map((c) => c.org.name)).toEqual(['Team Duburg', 'Team Jürgensby'])
  })

  it('returns null when the root id is unknown', () => {
    expect(buildTree(orgs, 'nope')).toBeNull()
  })

  it('drops nodes whose parent is missing', () => {
    const tree = buildTree(orgs, 'root')
    expect(flatten(tree!).map((o) => o.id)).not.toContain('orphan')
  })
})

describe('subtreeMemberCount', () => {
  it('sums the node and all descendants', () => {
    expect(subtreeMemberCount(buildTree(orgs, 'nord')!)).toBe(15)
  })

  it('returns the own count for a leaf', () => {
    expect(subtreeMemberCount(buildTree(orgs, 'duburg')!)).toBe(6)
  })
})

describe('pathTo', () => {
  it('returns the chain from the root down to the target', () => {
    const path = pathTo(buildTree(orgs, 'root')!, 'duburg')
    expect(path.map((o) => o.id)).toEqual(['root', 'amt', 'nord', 'duburg'])
  })

  it('returns just the root when the target is the root', () => {
    const path = pathTo(buildTree(orgs, 'amt')!, 'amt')
    expect(path.map((o) => o.id)).toEqual(['amt'])
  })

  it('returns an empty array when the target is absent', () => {
    expect(pathTo(buildTree(orgs, 'amt')!, 'nope')).toEqual([])
  })
})
