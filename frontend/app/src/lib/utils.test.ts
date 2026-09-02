import { describe, it, expect } from 'vitest'
import { formatKm } from './utils'
import { switchLanguage } from '@/lib/i18n'

describe('formatKm', () => {
  it('converts meters to km with 2 decimals, using a German decimal comma by default', () => {
    expect(formatKm(16545)).toBe('16,55 km')
  })

  it('returns "0 km" for zero', () => {
    expect(formatKm(0)).toBe('0 km')
  })

  it('rounds sub-km values', () => {
    expect(formatKm(999)).toBe('1 km')
  })

  it('uses a decimal point in English', async () => {
    await switchLanguage('en')
    expect(formatKm(16545)).toBe('16.55 km')
    await switchLanguage('de')
  })
})
