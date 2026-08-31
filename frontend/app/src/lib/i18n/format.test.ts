import { describe, expect, it } from 'vitest'
import { dateFnsLocale, intlLocale } from './format'

describe('locale mapping', () => {
  it('maps German tags to de-DE and the date-fns de locale', () => {
    expect(intlLocale('de')).toBe('de-DE')
    expect(intlLocale('de-DE')).toBe('de-DE')
    expect(dateFnsLocale('de').code).toBe('de')
  })

  it('maps English tags to en-GB', () => {
    expect(intlLocale('en')).toBe('en-GB')
    expect(intlLocale('en-US')).toBe('en-GB')
    expect(dateFnsLocale('en').code).toBe('en-GB')
  })

  it('falls back to German for an unsupported tag', () => {
    expect(intlLocale('fr-FR')).toBe('de-DE')
    expect(dateFnsLocale('fr-FR').code).toBe('de')
  })
})
