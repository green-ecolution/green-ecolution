import { describe, expect, it } from 'vitest'
import { createI18n } from '@/lib/i18n'
import { TIME_WINDOW_KEYS, timeWindowOptions, windowStart, type TimeWindowKey } from './timeWindows'

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

  it('defines a label for every window', async () => {
    const i18n = await createI18n()
    const t = i18n.getFixedT('de', 'common')
    for (const key of TIME_WINDOW_KEYS as readonly TimeWindowKey[]) {
      expect(t(`chart.timeWindow.${key}`)).toBeTruthy()
    }
  })
})

describe('timeWindowOptions', () => {
  it('maps keys to value/label pairs', async () => {
    const i18n = await createI18n()
    const t = i18n.getFixedT('de', 'common')
    expect(timeWindowOptions(['24h', '7d'], t)).toEqual([
      { value: '24h', label: '24 h' },
      { value: '7d', label: '7 Tage' },
    ])
  })
})
