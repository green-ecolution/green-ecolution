import { describe, it, expect } from 'vitest'
import { initialsOf, initialsOfName } from './initials'

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

describe('initialsOfName', () => {
  it('takes the single initial of a one-word name', () => {
    expect(initialsOfName('Grünflächenamt')).toBe('G')
  })

  it('ignores leading and repeated internal spaces', () => {
    expect(initialsOfName('  Team   Duburg  Nord')).toBe('TD')
  })

  it('handles a non-ASCII first letter', () => {
    expect(initialsOfName('Ökologie Nord')).toBe('ÖN')
  })

  it('does not throw on a whitespace-only name', () => {
    expect(initialsOfName('   ')).toBe('')
  })
})
