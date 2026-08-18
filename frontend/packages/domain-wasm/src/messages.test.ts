import { describe, expect, it } from 'vitest'
import { translateIssue } from './messages'
import type { ValidationIssue } from './types'

const issue = (overrides: Partial<ValidationIssue>): ValidationIssue => ({
  path: 'species',
  field: 'tree.species',
  key: 'tree.species.empty',
  params: {},
  ...overrides,
})

describe('translateIssue', () => {
  it('returns the German message for tree.species.empty', () => {
    expect(translateIssue(issue({}))).toBe('Art ist erforderlich.')
  })

  it('returns the German message for tree.number.empty', () => {
    expect(
      translateIssue(issue({ path: 'number', field: 'tree.number', key: 'tree.number.empty' })),
    ).toBe('Baumnummer ist erforderlich.')
  })

  it('interpolates min/max for planting_year.outOfRange', () => {
    const text = translateIssue(
      issue({
        path: 'plantingYear',
        field: 'tree.planting_year',
        key: 'tree.planting_year.outOfRange',
        params: { min: 0, max: 2026, got: 9999 },
      }),
    )
    expect(text).toContain('2026')
  })

  it('falls back to the i18n key when no entry exists', () => {
    const text = translateIssue(issue({ key: 'unknown.key.empty', field: 'unknown.key' }))
    expect(text).toBe('unknown.key.empty')
  })

  it('returns the German message for phone_number.empty', () => {
    expect(
      translateIssue(
        issue({ path: 'phoneNumber', field: 'phone_number', key: 'phone_number.empty' }),
      ),
    ).toBe('Telefonnummer ist erforderlich.')
  })

  it('returns one message for phone_number.invalidFormat covering both letters and too few digits', () => {
    const text = translateIssue(
      issue({
        path: 'phoneNumber',
        field: 'phone_number',
        key: 'phone_number.invalidFormat',
        params: { reason: 'must contain only digits...' },
      }),
    )
    expect(text).toBe(
      'Telefonnummer ist ungültig. Erlaubt sind Ziffern, Leerzeichen sowie -, /, ( ) und ein führendes +, mit mindestens 6 Ziffern.',
    )
  })
})
