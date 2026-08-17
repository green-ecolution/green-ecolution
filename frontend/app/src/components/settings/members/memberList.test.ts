import { describe, it, expect } from 'vitest'
import type { RoleResponse, UserResponse } from '@/api/backendApi'
import { initialsOf } from '@/lib/initials'
import {
  emptyMessageOf,
  fullNameOf,
  isFiltered,
  memberCountLabel,
  roleOverflow,
} from './memberList'
import { sinceLabel } from './cardChrome'

const role = (id: string, name: string): RoleResponse => ({
  id,
  name,
  permissions: [],
  isTemplate: false,
})

const user = (firstName: string, lastName: string): UserResponse => ({
  firstName,
  lastName,
})

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

describe('roleOverflow', () => {
  it('shows everything below the limit', () => {
    const { visible, overflow } = roleOverflow([role('a', 'A'), role('b', 'B')], 2)
    expect(visible).toHaveLength(2)
    expect(overflow).toBe(0)
  })

  it('counts the remainder above the limit', () => {
    const { visible, overflow } = roleOverflow([role('a', 'A'), role('b', 'B'), role('c', 'C')], 2)
    expect(visible).toHaveLength(2)
    expect(overflow).toBe(1)
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
  it('distinguishes no hits from no members at all', () => {
    expect(emptyMessageOf(true)).not.toBe(emptyMessageOf(false))
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
