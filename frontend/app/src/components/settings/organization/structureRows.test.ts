import { describe, it, expect } from 'vitest'
import type { OrganizationResponse } from '@/api/backendApi'
import { structureRows } from './structureRows'

const ORGS: OrganizationResponse[] = [
  { id: 'nord', name: 'Stadtgärtnerei Nord', memberCount: 4, parentId: 'amt' },
  { id: 'amt', name: 'Grünflächenamt', memberCount: 3, parentId: 'root', contactPersonId: 'u1' },
  { id: 'duburg', name: 'Team Duburg', memberCount: 6, parentId: 'nord' },
  { id: 'sued', name: 'Betriebshof Süd', memberCount: 2, parentId: 'amt' },
]

const ACTIVE = new Map<string, readonly string[]>([
  ['amt', ['alpha', 'beta']],
  ['nord', ['alpha']],
])

describe('structureRows', () => {
  it('orders the organizations as an indented tree, siblings by name', () => {
    const rows = structureRows(ORGS, ACTIVE)
    expect(rows.map((row) => [row.name, row.depth])).toEqual([
      ['Grünflächenamt', 0],
      ['Betriebshof Süd', 1],
      ['Stadtgärtnerei Nord', 1],
      ['Team Duburg', 2],
    ])
  })

  it('treats an organization whose parent is not visible as a top row', () => {
    const rows = structureRows(ORGS, ACTIVE)
    expect(rows[0]).toMatchObject({ id: 'amt', depth: 0 })
  })

  it('counts direct members and the whole subtree', () => {
    const rows = structureRows(ORGS, ACTIVE)
    expect(rows.find((row) => row.id === 'amt')).toMatchObject({ memberDirect: 3, memberTotal: 15 })
    expect(rows.find((row) => row.id === 'nord')).toMatchObject({
      memberDirect: 4,
      memberTotal: 10,
    })
  })

  it('carries the contact person and the active modules', () => {
    const rows = structureRows(ORGS, ACTIVE)
    expect(rows.find((row) => row.id === 'amt')).toMatchObject({
      contactPersonId: 'u1',
      moduleKeys: ['alpha', 'beta'],
    })
    expect(rows.find((row) => row.id === 'sued')).toMatchObject({
      contactPersonId: null,
      moduleKeys: [],
    })
  })
})
