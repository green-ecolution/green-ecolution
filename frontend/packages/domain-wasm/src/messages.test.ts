import { describe, expect, it } from 'vitest'
import { VALIDATION_KEYS, translateIssue } from './messages'

describe('translateIssue', () => {
  it('delegates to the injected translator', () => {
    const result = translateIssue(
      { key: 'tree.species.tooShort', params: { min: 3 } },
      (key, params) => `${key}|${JSON.stringify(params)}`,
    )
    expect(result).toBe('tree.species.tooShort|{"min":3}')
  })

  it('falls back to the raw key when the translator returns nothing usable', () => {
    expect(translateIssue({ key: 'tree.species.empty', params: {} }, () => '')).toBe(
      'tree.species.empty',
    )
  })

  it('exports every key the validators can emit', () => {
    expect(VALIDATION_KEYS).toContain('tree.species.empty')
    expect(VALIDATION_KEYS).toContain('user.phone_number.invalidFormat')
    expect(new Set(VALIDATION_KEYS).size).toBe(VALIDATION_KEYS.length)
  })

  it('holds no German text any more', () => {
    for (const key of VALIDATION_KEYS) {
      expect(key).not.toMatch(/[äöüßÄÖÜ]/)
      expect(key).toMatch(/^[a-z][a-zA-Z0-9_.]*$/)
    }
  })
})
