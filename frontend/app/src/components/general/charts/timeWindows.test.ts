import { describe, expect, it } from 'vitest'
import { TIME_WINDOWS, timeWindowOptions, windowStart, type TimeWindowKey } from './timeWindows'

describe('windowStart', () => {
  const now = new Date('2026-07-13T14:37:22.123Z').getTime()

  it('returns undefined for the unbounded window', () => {
    expect(windowStart('all', now)).toBeUndefined()
  })

  it('subtracts the window size from the hour-truncated now', () => {
    expect(windowStart('24h', now)?.toISOString()).toBe('2026-07-12T14:00:00.000Z')
    expect(windowStart('7d', now)?.toISOString()).toBe('2026-07-06T14:00:00.000Z')
    expect(windowStart('30d', now)?.toISOString()).toBe('2026-06-13T14:00:00.000Z')
  })

  it('defines a label for every window', () => {
    for (const key of Object.keys(TIME_WINDOWS) as TimeWindowKey[]) {
      expect(TIME_WINDOWS[key].label).toBeTruthy()
    }
  })
})

describe('timeWindowOptions', () => {
  it('maps keys to value/label pairs', () => {
    expect(timeWindowOptions(['24h', '7d'])).toEqual([
      { value: '24h', label: '24 h' },
      { value: '7d', label: '7 Tage' },
    ])
  })
})
