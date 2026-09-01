import type { TFunction } from 'i18next'

export const TIME_WINDOW_KEYS = ['24h', '7d', '30d', 'all'] as const

export type TimeWindowKey = (typeof TIME_WINDOW_KEYS)[number]

const WINDOW_DAYS: Record<TimeWindowKey, number | null> = {
  '24h': 1,
  '7d': 7,
  '30d': 30,
  all: null,
}

const HOUR_MS = 60 * 60 * 1000
const DAY_MS = 24 * HOUR_MS

// Truncated to the full hour so the TanStack query key stays stable across
// re-renders instead of minting a new key (and refetch) on every render.
export const windowStart = (key: TimeWindowKey, now: number): Date | undefined => {
  const days = WINDOW_DAYS[key]
  if (days === null) return undefined
  return new Date(Math.floor(now / HOUR_MS) * HOUR_MS - days * DAY_MS)
}

export const timeWindowOptions = <K extends TimeWindowKey>(keys: K[], t: TFunction<'common'>) =>
  keys.map((key) => ({ value: key, label: t(`chart.timeWindow.${key}`) }))
