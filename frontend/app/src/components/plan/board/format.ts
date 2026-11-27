import { format, type Locale } from 'date-fns'
import type { User } from '@/api/backendApi'

export function userInitials(user: User): string {
  return `${user.firstName.charAt(0)}${user.lastName.charAt(0)}`.toUpperCase()
}

// Not a component: the caller resolves the active locale via `useDateLocale()`
// and passes it in, rather than this module reaching for `getI18n()` — a
// frozen locale here would reproduce the router-helper bug fixed earlier.
export function formatBoardDate(iso: string, locale: Locale): string {
  const date = new Date(iso)
  const sameYear = date.getFullYear() === new Date().getFullYear()
  return format(date, sameYear ? 'EEEEEE, d. MMMM' : 'EEEEEE, d. MMMM yyyy', { locale })
}
