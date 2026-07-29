import { describe, it, expect } from 'vitest'
import type { OrganizationResponse } from '@/api/backendApi'
import { buildTree, flatten, pathTo, subtreeMemberCount } from './organizationTree'

const org = (
  id: string,
  parentId: string | null | undefined,
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
  org('root', undefined, 'Green Ecolution'),
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

  it('handles null parentId (production API case)', () => {
    const withNull = [org('root', null, 'Green Ecolution'), org('child', 'root', 'Child')]
    const tree = buildTree(withNull, 'root')
    expect(tree?.org.id).toBe('root')
    expect(tree?.children.map((c) => c.org.id)).toEqual(['child'])
  })

  it('handles self-referential cycle without hanging', () => {
    const selfRef = [org('root', undefined, 'Root'), org('self', 'self', 'Self Reference')]
    const tree = buildTree(selfRef, 'root')
    expect(tree?.org.id).toBe('root')
    expect(tree?.children).toEqual([])
  })

  it('handles mutual cycle without hanging', () => {
    const mutual = [
      org('root', undefined, 'Root'),
      org('a', 'b', 'Node A'),
      org('b', 'a', 'Node B'),
    ]
    const tree = buildTree(mutual, 'root')
    expect(tree?.org.id).toBe('root')
    expect(tree?.children).toEqual([])
  })

  it('sorts siblings by German collation', () => {
    const deCollation = [
      org('root', undefined, 'Root'),
      org('p', 'root', 'Pfirsich'),
      org('o', 'root', 'Öl'),
      org('n', 'root', 'Nuss'),
    ]
    const tree = buildTree(deCollation, 'root')
    const sorted = tree?.children.map((c) => c.org.name)
    expect(sorted).toEqual(['Nuss', 'Öl', 'Pfirsich'])
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
