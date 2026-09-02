import { Search } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Combobox, Input, Loading, SelectField, SimplePagination } from '@green-ecolution/ui'
import type { OrganizationResponse, RoleResponse, UserResponse } from '@/api/backendApi'
import MemberListItem from './MemberListItem'
import { emptyMessageOf, isFiltered, memberCountLabel, roleFilterOptions } from './memberList'

const ALL = '__all__'

interface MemberListProps {
  members: UserResponse[]
  /** From the response's `pagination.totalRecords` — the backend counts, not us. */
  total: number
  page: number
  totalPages: number
  loading: boolean
  selectedId: string | null
  search: string
  organizationFilter: string | null
  roleFilter: string | null
  organizations: OrganizationResponse[]
  roles: RoleResponse[]
  onSearchChange: (value: string) => void
  onOrganizationFilterChange: (orgId: string | null) => void
  onRoleFilterChange: (roleId: string | null) => void
  onPageChange: (page: number) => void
  onSelect: (user: UserResponse) => void
}

const MemberList = ({
  members,
  total,
  page,
  totalPages,
  loading,
  selectedId,
  search,
  organizationFilter,
  roleFilter,
  organizations,
  roles,
  onSearchChange,
  onOrganizationFilterChange,
  onRoleFilterChange,
  onPageChange,
  onSelect,
}: MemberListProps) => {
  const { t } = useTranslation('settings')
  const filtered = isFiltered(search, organizationFilter, roleFilter)

  // With an organization picked, only its own roles can ever match: the backend
  // requires a role's organization to equal its holder's. Offering the rest
  // would let the user build a combination that is empty by construction.
  const selectableRoles =
    organizationFilter === null
      ? roles
      : roles.filter((role) => role.organizationId === organizationFilter)

  return (
    <div className="flex flex-col gap-4">
      <h2 className="font-lato text-base font-semibold">{t('members.listTitle')}</h2>

      <div className="relative">
        <Search
          className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-dark-400"
          aria-hidden
        />
        <Input
          type="search"
          value={search}
          onChange={(event) => onSearchChange(event.target.value)}
          placeholder={t('members.searchPlaceholder')}
          aria-label={t('members.searchAriaLabel')}
          className="pl-9"
        />
      </div>

      <SelectField
        label={t('members.organizationFilterLabel')}
        hideLabel
        placeholder={t('members.organizationFilterPlaceholder')}
        value={organizationFilter ?? ALL}
        onValueChange={(value) => onOrganizationFilterChange(value === ALL ? null : value)}
        options={[
          { value: ALL, label: t('members.organizationFilterAllLabel') },
          ...organizations.map((org) => ({ value: org.id, label: org.name })),
        ]}
      />

      <Combobox
        aria-label={t('members.roleFilterAriaLabel')}
        placeholder={t('members.roleFilterPlaceholder')}
        searchPlaceholder={t('members.roleFilterSearchPlaceholder')}
        emptyText={t('members.roleFilterEmptyText')}
        value={roleFilter ?? ALL}
        onChange={(value) => onRoleFilterChange(value === ALL ? null : value)}
        options={[
          { value: ALL, label: t('members.roleFilterAllLabel') },
          ...roleFilterOptions(selectableRoles, organizations, t),
        ]}
      />

      <p className="text-xs font-semibold uppercase tracking-wider text-dark-500">
        {memberCountLabel(members.length, total, filtered, t)}
      </p>

      {loading ? (
        <Loading className="justify-center py-6" label={t('members.loading')} />
      ) : members.length === 0 ? (
        <p className="text-sm text-dark-600">{emptyMessageOf(filtered, t)}</p>
      ) : (
        <ul
          role="list"
          aria-label={t('members.listAriaLabel')}
          className="flex list-none flex-col gap-2"
        >
          {members.map((user) => (
            <li key={user.id}>
              <MemberListItem
                user={user}
                selected={user.id === selectedId}
                onSelect={() => onSelect(user)}
              />
            </li>
          ))}
        </ul>
      )}

      {totalPages > 1 && (
        <SimplePagination
          pagination={{
            currentPage: page,
            totalPages,
            hasNextPage: page < totalPages,
            hasPreviousPage: page > 1,
          }}
          onPageChange={onPageChange}
        />
      )}
    </div>
  )
}

export default MemberList
