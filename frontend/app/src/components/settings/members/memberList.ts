import type { TFunction } from 'i18next'
import type { ComboboxOption } from '@green-ecolution/ui'
import { validatePhoneNumber, type ValidationIssue } from '@green-ecolution/domain-wasm'
import type { OrganizationResponse, RoleResponse, UserResponse } from '@/api/backendApi'
import { roleDisplayName } from '../roles/roleList'

export const fullNameOf = (user: UserResponse): string =>
  [user.firstName, user.lastName].filter((part) => (part ?? '').trim().length > 0).join(' ')

export const memberCountLabel = (
  shown: number,
  total: number,
  filtered: boolean,
  t: TFunction<'settings'>,
): string =>
  filtered
    ? t('members.countFiltered', { shown, total, noun: t('members.personNoun', { count: total }) })
    : t('members.count', { count: total })

export const emptyMessageOf = (filtered: boolean, t: TFunction<'settings'>): string =>
  t(filtered ? 'members.emptyFiltered' : 'members.emptyUnfiltered')

export const isFiltered = (
  search: string,
  organizationId: string | null,
  roleId: string | null,
): boolean => search.trim().length > 0 || organizationId !== null || roleId !== null

/**
 * The phone number field is optional, but `PhoneNumber::new("")` is an error —
 * emptiness is a property of this field, not of the value object — so an
 * empty value skips validation here rather than in the domain rule.
 */
export const phoneNumberIssue = (value: string): ValidationIssue | null =>
  value.trim().length === 0 ? null : (validatePhoneNumber(value) as ValidationIssue | null)

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
  t: TFunction<'settings'>,
): ComboboxOption[] => {
  const known = new Set(organizations.map((org) => org.id))
  const grouped = organizations.flatMap((org) =>
    roles
      .filter((role) => role.organizationId === org.id)
      .map((role) => ({ value: role.id, label: roleDisplayName(role, t), group: org.name })),
  )
  // Without this a role whose organization is not in the visible list would
  // disappear from the filter with no trace.
  const orphans = roles
    .filter((role) => role.organizationId == null || !known.has(role.organizationId))
    .map((role) => ({
      value: role.id,
      label: roleDisplayName(role, t),
      group: t('members.roleFilterOrphanGroup'),
    }))
  return [...grouped, ...orphans]
}
