import { describe, it, expect } from 'vitest'
import { expandShortYear } from './plantingYear'

const REFERENCE_YEAR = 2026

describe('expandShortYear', () => {
  it('expands a two-digit year up to the reference year into this century', () => {
    expect(expandShortYear(25, REFERENCE_YEAR)).toBe(2025)
    expect(expandShortYear(26, REFERENCE_YEAR)).toBe(2026)
  })

  // Plantings are scheduled ahead, so a shorthand just past today means 20xx.
  it('expands a two-digit year inside the planning window into this century', () => {
    expect(expandShortYear(27, REFERENCE_YEAR)).toBe(2027)
    expect(expandShortYear(29, REFERENCE_YEAR)).toBe(2029)
    expect(expandShortYear(46, REFERENCE_YEAR)).toBe(2046)
  })

  // Beyond the window a shorthand is far more likely to mean an old tree than a
  // planting seventy years out.
  it('expands a two-digit year beyond the planning window into the last century', () => {
    expect(expandShortYear(47, REFERENCE_YEAR)).toBe(1947)
    expect(expandShortYear(85, REFERENCE_YEAR)).toBe(1985)
    expect(expandShortYear(99, REFERENCE_YEAR)).toBe(1999)
  })

  it('expands a one-digit year', () => {
    expect(expandShortYear(0, REFERENCE_YEAR)).toBe(2000)
    expect(expandShortYear(5, REFERENCE_YEAR)).toBe(2005)
  })

  it('leaves a four-digit year untouched', () => {
    expect(expandShortYear(2025, REFERENCE_YEAR)).toBe(2025)
    expect(expandShortYear(1985, REFERENCE_YEAR)).toBe(1985)
  })

  // Guessing here would turn a typo into a plausible-looking year.
  it('leaves a three-digit year untouched so it fails validation', () => {
    expect(expandShortYear(202, REFERENCE_YEAR)).toBe(202)
  })

  it('leaves negative and fractional input untouched', () => {
    expect(expandShortYear(-25, REFERENCE_YEAR)).toBe(-25)
    expect(expandShortYear(25.5, REFERENCE_YEAR)).toBe(25.5)
  })

  it('passes NaN through for an empty field', () => {
    expect(expandShortYear(Number.NaN, REFERENCE_YEAR)).toBeNaN()
  })

  it('defaults to the current year as reference', () => {
    const currentShortYear = new Date().getFullYear() % 100
    expect(expandShortYear(currentShortYear)).toBe(new Date().getFullYear())
  })
})
