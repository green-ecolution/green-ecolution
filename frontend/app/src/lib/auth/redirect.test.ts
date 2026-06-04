import { describe, it, expect } from 'vitest'
import { sanitizeReturnTo } from './redirect'

describe('sanitizeReturnTo', () => {
  it('keeps a normal internal path', () => {
    expect(sanitizeReturnTo('/dashboard')).toBe('/dashboard')
    expect(sanitizeReturnTo('/trees?page=2')).toBe('/trees?page=2')
  })

  it('falls back to /dashboard for empty or non-path input', () => {
    expect(sanitizeReturnTo(undefined)).toBe('/dashboard')
    expect(sanitizeReturnTo('')).toBe('/dashboard')
    expect(sanitizeReturnTo('dashboard')).toBe('/dashboard')
  })

  it('rejects protocol-relative and absolute URLs (open-redirect guard)', () => {
    expect(sanitizeReturnTo('//evil.com')).toBe('/dashboard')
    expect(sanitizeReturnTo('https://evil.com')).toBe('/dashboard')
    expect(sanitizeReturnTo('/\\evil.com')).toBe('/dashboard')
  })
})
