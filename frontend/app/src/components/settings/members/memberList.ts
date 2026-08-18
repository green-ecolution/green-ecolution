import type { ComboboxOption } from '@green-ecolution/ui'
import type { OrganizationResponse, RoleResponse, UserResponse } from '@/api/backendApi'

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

/**
 * Mirrors the backend's `PhoneNumber` value object exactly (allowed: digits,
 * space, `-`, `/`, `(`, `)`, plus a leading `+`; at least 6 digits). If the two
 * ever drift apart, a person could pass this check and still get a 400 from
 * the server with no way to tell why.
 */
export const phoneNumberError = (value: string): string | null => {
  const trimmed = value.trim()
  if (trimmed.length === 0) return null

  let digitCount = 0
  for (let i = 0; i < trimmed.length; i += 1) {
    const char = trimmed[i]
    if (char >= '0' && char <= '9') {
      digitCount += 1
    } else if (char === ' ' || char === '-' || char === '/' || char === '(' || char === ')') {
      continue
    } else if (char === '+' && i === 0) {
      continue
    } else {
      return 'Enthält Zeichen, die in einer Telefonnummer nicht vorkommen.'
    }
  }

  return digitCount >= 6 ? null : 'Das sind zu wenige Ziffern für eine Telefonnummer.'
}

export const sinceLabel = (createdAt?: string | null): string | null => {
  if (createdAt == null) return null
  const date = new Date(createdAt)
  if (Number.isNaN(date.getTime())) return null
  return `${String(date.getMonth() + 1).padStart(2, '0')}/${date.getFullYear()}`
}

/**
 * Creating an organization instantiates org-owned copies of every role template,
 * so role names repeat across a subtree. Grouping by organization is what makes
 * four roles called "Administrator" tellable apart.
 */
export const roleFilterOptions = (
  roles: RoleResponse[],
  organizations: OrganizationResponse[],
): ComboboxOption[] => {
  const known = new Set(organizations.map((org) => org.id))
  const grouped = organizations.flatMap((org) =>
    roles
      .filter((role) => role.organizationId === org.id)
      .map((role) => ({ value: role.id, label: role.name, group: org.name })),
  )
  // Without this a role whose organization is not in the visible list would
  // disappear from the filter with no trace.
  const orphans = roles
    .filter((role) => role.organizationId == null || !known.has(role.organizationId))
    .map((role) => ({ value: role.id, label: role.name, group: 'Ohne Organisation' }))
  return [...grouped, ...orphans]
}
