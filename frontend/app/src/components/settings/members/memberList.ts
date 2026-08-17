import type { UserResponse } from '@/api/backendApi'

export const fullNameOf = (user: UserResponse): string =>
  [user.firstName, user.lastName].filter((part) => (part ?? '').trim().length > 0).join(' ')

const personNoun = (count: number): string => (count === 1 ? 'Person' : 'Personen')

export const memberCountLabel = (shown: number, total: number, filtered: boolean): string =>
  filtered ? `${shown} von ${total} ${personNoun(total)}` : `${total} ${personNoun(total)}`

export const emptyMessageOf = (filtered: boolean): string =>
  filtered
    ? 'Keine Person passt zu Suche und Filter.'
    : 'Es sind noch keine Mitarbeitenden hinterlegt.'

export const isFiltered = (
  search: string,
  organizationId: string | null,
  roleId: string | null,
): boolean => search.trim().length > 0 || organizationId !== null || roleId !== null

export const sinceLabel = (createdAt?: string | null): string | null => {
  if (createdAt == null) return null
  const date = new Date(createdAt)
  if (Number.isNaN(date.getTime())) return null
  return `${String(date.getMonth() + 1).padStart(2, '0')}/${date.getFullYear()}`
}
