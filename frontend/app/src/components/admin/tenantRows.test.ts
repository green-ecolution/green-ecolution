import { describe, expect, it } from 'vitest'
import type { OrganizationResponse } from '@green-ecolution/backend-client'
import { tenantRows as tenantRowsOrNull } from './tenantRows'

const org = (
  id: string,
  name: string,
  parentId: string | null,
  memberCount = 0,
): OrganizationResponse => ({ id, name, parentId, memberCount })

const ROOT = org('root', 'Green Ecolution', null, 2)
const TBZ = org('tbz', 'TBZ', 'root', 5)
const EXTERN_A = org('extern-a', 'Extern A', 'tbz', 3)
const EXTERN_A1 = org('extern-a1', 'Extern A1', 'extern-a', 1)
const ANDERE = org('andere', 'Andere Stadt', 'root', 4)

const ORGS = [ROOT, TBZ, EXTERN_A, EXTERN_A1, ANDERE]

const tenantRows = (...args: Parameters<typeof tenantRowsOrNull>) => {
  const rows = tenantRowsOrNull(...args)
  if (rows === null) throw new Error('expected rows')
  return rows
}

describe('tenantRows', () => {
  it('turns only the direct children of the root into rows, sorted by name', () => {
    const rows = tenantRows(ORGS, undefined, undefined)
    expect(rows.map((row) => row.id)).toEqual(['andere', 'tbz'])
  })

  it('counts descendants and members over the subtree', () => {
    const [, tbz] = tenantRows(ORGS, undefined, undefined)
    expect(tbz.subOrganizationCount).toBe(2)
    expect(tbz.memberCount).toBe(9)
  })

  it('takes the active modules and the counts of the tenant itself', () => {
    const rows = tenantRows(
      ORGS,
      {
        modules: ['irrigation', 'waste'],
        organizations: [
          { organizationId: 'tbz', active: ['irrigation'] },
          { organizationId: 'extern-a', active: ['irrigation', 'waste'] },
        ],
      },
      [{ organizationId: 'tbz', sensorCount: 7, vehicleCount: 2, planCount: 11 }],
    )
    const tbz = rows.find((row) => row.id === 'tbz')
    expect(tbz).toMatchObject({
      activeModules: ['irrigation'],
      sensorCount: 7,
      vehicleCount: 2,
      planCount: 11,
    })
  })

  it('reports missing counts as null, not zero', () => {
    const [andere] = tenantRows(ORGS, undefined, undefined)
    expect(andere).toMatchObject({
      activeModules: null,
      sensorCount: null,
      vehicleCount: null,
      planCount: null,
    })
  })

  it('reports an empty module list once modules are loaded and the tenant has none', () => {
    const [andere] = tenantRows(
      ORGS,
      {
        modules: ['irrigation'],
        organizations: [{ organizationId: 'tbz', active: ['irrigation'] }],
      },
      undefined,
    )
    expect(andere.activeModules).toEqual([])
  })

  it('is null without a root in the list', () => {
    expect(tenantRowsOrNull([TBZ, EXTERN_A], undefined, undefined)).toBeNull()
  })

  it('gives an empty list for a root without children', () => {
    expect(tenantRows([ROOT], undefined, undefined)).toEqual([])
  })
})
