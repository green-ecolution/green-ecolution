export const TIME_WINDOWS = {
  '24h': { label: '24 h', days: 1 },
  '7d': { label: '7 Tage', days: 7 },
  '30d': { label: '30 Tage', days: 30 },
  all: { label: 'Alles', days: null },
} as const

export type TimeWindowKey = keyof typeof TIME_WINDOWS

const HOUR_MS = 60 * 60 * 1000
const DAY_MS = 24 * HOUR_MS

// Truncated to the full hour so the TanStack query key stays stable across
// re-renders instead of minting a new key (and refetch) on every render.
export const windowStart = (key: TimeWindowKey, now: number): Date | undefined => {
  const { days } = TIME_WINDOWS[key]
  if (days === null) return undefined
  return new Date(Math.floor(now / HOUR_MS) * HOUR_MS - days * DAY_MS)
}

export const timeWindowOptions = (keys: TimeWindowKey[]) =>
  keys.map((key) => ({ value: key, label: TIME_WINDOWS[key].label }))
