import { format } from 'date-fns'
import { de } from 'date-fns/locale'
import type { User } from '@/api/backendApi'

export function userInitials(user: User): string {
  return `${user.firstName.charAt(0)}${user.lastName.charAt(0)}`.toUpperCase()
}

export function formatBoardDate(iso: string): string {
  const date = new Date(iso)
  const sameYear = date.getFullYear() === new Date().getFullYear()
  return format(date, sameYear ? 'EEEEEE, d. MMMM' : 'EEEEEE, d. MMMM yyyy', { locale: de })
}
