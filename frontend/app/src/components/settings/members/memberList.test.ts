import { describe, it, expect } from 'vitest'
import type { OrganizationResponse, RoleResponse, UserResponse } from '@/api/backendApi'
import { initialsOf } from '@/lib/initials'
import {
  emptyMessageOf,
  fullNameOf,
  isFiltered,
  memberCountLabel,
  roleFilterOptions,
  sinceLabel,
} from './memberList'

const user = (firstName: string, lastName: string): UserResponse =>
  ({ firstName, lastName }) as UserResponse

describe('initialsOf', () => {
  it('takes the first letter of each name', () => {
    expect(initialsOf('Anna', 'Ahlmann')).toBe('AA')
  })

  it('copes with a missing last name', () => {
    expect(initialsOf('Anna', null)).toBe('A')
  })

  it('returns an empty string when nothing is known', () => {
    expect(initialsOf(null, undefined)).toBe('')
  })
})

describe('fullNameOf', () => {
  it('joins first and last name', () => {
    expect(fullNameOf(user('Anna', 'Ahlmann'))).toBe('Anna Ahlmann')
  })

  it('does not leave a stray space when a part is missing', () => {
    expect(fullNameOf(user('Anna', ''))).toBe('Anna')
  })
})

describe('memberCountLabel', () => {
  it('names one number without a filter', () => {
    expect(memberCountLabel(3, 3, false)).toBe('3 Personen')
  })

  it('uses the singular for one person', () => {
    expect(memberCountLabel(1, 1, false)).toBe('1 Person')
  })

  it('names both numbers with an active filter, so the narrowing stays visible', () => {
    expect(memberCountLabel(2, 7, true)).toBe('2 von 7 Personen')
  })
})

describe('emptyMessageOf', () => {
  it('says "no search hits" when filtered', () => {
    expect(emptyMessageOf(true)).toBe('Keine Person passt zu Suche und Filter.')
  })

  it('says "no members yet" when not filtered', () => {
    expect(emptyMessageOf(false)).toBe('Es sind noch keine Mitarbeitenden hinterlegt.')
  })
})

describe('isFiltered', () => {
  it('ignores a blank search', () => {
    expect(isFiltered('   ', null, null)).toBe(false)
  })

  it('reports any active narrowing', () => {
    expect(isFiltered('', 'org-1', null)).toBe(true)
    expect(isFiltered('anna', null, null)).toBe(true)
  })
})

describe('sinceLabel', () => {
  it('formats an ISO timestamp as MM/YYYY', () => {
    expect(sinceLabel('2026-08-17T14:30:00.000Z')).toBe('08/2026')
  })

  it('returns null for a missing value', () => {
    expect(sinceLabel(null)).toBe(null)
    expect(sinceLabel(undefined)).toBe(null)
  })

  it('returns null for an unparseable value', () => {
    expect(sinceLabel('not a date')).toBe(null)
  })
})

describe('roleFilterOptions', () => {
  const org = (id: string, name: string): OrganizationResponse =>
    ({ id, name }) as OrganizationResponse
  const role = (id: string, name: string, organizationId: string | null): RoleResponse =>
    ({ id, name, organizationId }) as RoleResponse

  const orgs = [org('amt', 'Grünflächenamt'), org('nord', 'Stadtgärtnerei Nord')]

  it('keeps same-named roles apart by grouping them under their organization', () => {
    const options = roleFilterOptions(
      [role('r1', 'Administrator', 'amt'), role('r2', 'Administrator', 'nord')],
      orgs,
    )

    expect(options).toEqual([
      { value: 'r1', label: 'Administrator', group: 'Grünflächenamt' },
      { value: 'r2', label: 'Administrator', group: 'Stadtgärtnerei Nord' },
    ])
  })

  it('emits groups in the order the organizations were given', () => {
    const options = roleFilterOptions(
      [role('r2', 'Gärtner', 'nord'), role('r1', 'Gärtner', 'amt')],
      orgs,
    )

    expect(options.map((option) => option.group)).toEqual(['Grünflächenamt', 'Stadtgärtnerei Nord'])
  })

  it('still lists a role whose organization is not visible instead of dropping it', () => {
    const options = roleFilterOptions([role('r9', 'Fremd', 'unbekannt')], orgs)

    expect(options).toEqual([{ value: 'r9', label: 'Fremd', group: 'Ohne Organisation' }])
  })

  it('yields nothing when there are no roles', () => {
    expect(roleFilterOptions([], orgs)).toEqual([])
  })
})
