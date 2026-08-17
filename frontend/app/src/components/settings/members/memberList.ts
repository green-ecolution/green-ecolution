import type { RoleResponse, UserResponse } from '@/api/backendApi'

const DEFAULT_VISIBLE_ROLES = 2

export const fullNameOf = (user: UserResponse): string =>
  [user.firstName, user.lastName].filter((part) => (part ?? '').trim().length > 0).join(' ')

export const roleOverflow = (
  roles: RoleResponse[],
  max: number = DEFAULT_VISIBLE_ROLES,
): { visible: RoleResponse[]; overflow: number } => ({
  visible: roles.slice(0, max),
  overflow: Math.max(0, roles.length - max),
})

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
