import { describe, it, expect, beforeAll } from 'vitest'
import type { TFunction } from 'i18next'
import { getI18n } from '@/lib/i18n'
import type { OrganizationResponse } from '@/api/backendApi'
import { buildTree } from './organizationTree'
import { directChildrenLine, memberSubtitle, memberSummary } from './organizationLabels'

let t: TFunction<'settings'>
beforeAll(() => {
  t = getI18n().getFixedT('de', 'settings')
})

const org = (
  id: string,
  parentId: string | undefined,
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

const treeOf = (orgs: OrganizationResponse[]) => buildTree(orgs, 'amt')!

describe('memberSubtitle', () => {
  it('sums the whole subtree', () => {
    const node = treeOf([
      org('amt', undefined, 'Grünflächenamt', 2),
      org('nord', 'amt', 'Nord', 3),
      org('sued', 'amt', 'Süd', 4),
    ])
    expect(memberSubtitle(node, t)).toBe('9 Personen')
  })

  it('uses the singular for a single person', () => {
    expect(memberSubtitle(treeOf([org('amt', undefined, 'Grünflächenamt', 1)]), t)).toBe('1 Person')
  })

  it('returns null for an empty subtree, so the row stays single-line', () => {
    const node = treeOf([org('amt', undefined, 'Grünflächenamt', 0), org('nord', 'amt', 'Nord', 0)])
    expect(memberSubtitle(node, t)).toBeNull()
  })
})

describe('memberSummary', () => {
  it('names the sub-organizations the people are spread over', () => {
    const node = treeOf([
      org('amt', undefined, 'Grünflächenamt', 2),
      org('nord', 'amt', 'Nord', 3),
      org('sued', 'amt', 'Süd', 4),
    ])
    expect(memberSummary(node, t)).toBe('9 Personen in 2 Unterorganisationen')
  })

  it('leaves out the sub-organizations when there are none', () => {
    expect(memberSummary(treeOf([org('amt', undefined, 'Grünflächenamt', 3)]), t)).toBe(
      '3 Personen',
    )
  })

  it('says nobody is assigned instead of counting empty sub-organizations', () => {
    const node = treeOf([
      org('amt', undefined, 'Grünflächenamt', 0),
      org('nord', 'amt', 'Nord', 0),
      org('sued', 'amt', 'Süd', 0),
    ])
    expect(memberSummary(node, t)).toBe('Keine Personen zugeordnet')
  })
})

describe('directChildrenLine', () => {
  it('uses the singular for one sub-organization', () => {
    expect(directChildrenLine(1, t)).toBe('1 direkte Unterorganisation')
  })

  it('uses the plural for several', () => {
    expect(directChildrenLine(3, t)).toBe('3 direkte Unterorganisationen')
  })
})
