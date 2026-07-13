export const SIGNAL_WINDOWS = {
  '24h': { label: '24h', days: 1 },
  '7d': { label: '7 Tage', days: 7 },
  '30d': { label: '30 Tage', days: 30 },
  all: { label: 'Alles', days: null },
} as const

export type SignalWindowKey = keyof typeof SIGNAL_WINDOWS

const HOUR_MS = 60 * 60 * 1000
const DAY_MS = 24 * HOUR_MS

// Truncated to the full hour so the TanStack query key stays stable across
// re-renders instead of minting a new key (and refetch) on every render.
export const windowStart = (key: SignalWindowKey, now: number): Date | undefined => {
  const { days } = SIGNAL_WINDOWS[key]
  if (days === null) return undefined
  return new Date(Math.floor(now / HOUR_MS) * HOUR_MS - days * DAY_MS)
}
