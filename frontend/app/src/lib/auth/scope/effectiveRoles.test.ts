import { describe, it, expect } from 'vitest'
import type { ReachableOrganizationResponse, RoleResponse } from '@green-ecolution/backend-client'
import { effectiveRoles } from './effectiveRoles'

const reachable: ReachableOrganizationResponse[] = [
  { id: 'root', name: 'Root', parentId: null },
  { id: 'tbz', name: 'TBZ', parentId: 'root' },
  { id: 'firma_a', name: 'Firma A', parentId: 'tbz' },
  { id: 'firma_a1', name: 'Firma A1', parentId: 'firma_a' },
  { id: 'firma_b', name: 'Firma B', parentId: 'tbz' },
]

const role = (organizationId: string | null): RoleResponse =>
  ({
    id: `r-${organizationId}`,
    name: 'Rolle',
    organizationId,
    permissions: ['tree:read'],
  }) as RoleResponse

describe('effectiveRoles', () => {
  it('returns the roles unchanged without a scope', () => {
    const roles = [role('tbz'), role(null)]
    expect(effectiveRoles(roles, null, reachable)).toBe(roles)
  })

  it('moves a role held above the scope onto the scope organization', () => {
    const result = effectiveRoles([role('tbz')], 'firma_a', reachable)
    expect(result).toHaveLength(1)
    expect(result[0].organizationId).toBe('firma_a')
  })

  it('keeps a role held on the scope itself', () => {
    expect(effectiveRoles([role('firma_a')], 'firma_a', reachable)[0].organizationId).toBe(
      'firma_a',
    )
  })

  it('keeps a role held below the scope', () => {
    expect(effectiveRoles([role('firma_a1')], 'firma_a', reachable)).toEqual([role('firma_a1')])
  })

  it('drops a role of an unrelated organization', () => {
    expect(effectiveRoles([role('firma_b')], 'firma_a', reachable)).toEqual([])
  })

  it('drops templates', () => {
    expect(effectiveRoles([role(null)], 'firma_a', reachable)).toEqual([])
  })
})
