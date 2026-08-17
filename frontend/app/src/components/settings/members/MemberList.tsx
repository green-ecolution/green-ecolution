import { Search } from 'lucide-react'
import { Input, Loading, SelectField, SimplePagination } from '@green-ecolution/ui'
import type { OrganizationResponse, RoleResponse, UserResponse } from '@/api/backendApi'
import MemberListItem from './MemberListItem'
import { emptyMessageOf, isFiltered, memberCountLabel } from './memberList'

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
  const filtered = isFiltered(search, organizationFilter, roleFilter)

  return (
    <div className="flex flex-col gap-4">
      <h2 className="font-lato text-base font-semibold">Mitarbeitende</h2>

      <div className="relative">
        <Search
          className="pointer-events-none absolute left-3 top-1/2 size-4 -translate-y-1/2 text-dark-400"
          aria-hidden
        />
        <Input
          type="search"
          value={search}
          onChange={(event) => onSearchChange(event.target.value)}
          placeholder="Name, Username oder E-Mail"
          aria-label="Mitarbeitende suchen"
          className="pl-9"
        />
      </div>

      <SelectField
        label="Organisation"
        hideLabel
        placeholder="Alle Organisationen"
        value={organizationFilter ?? ALL}
        onValueChange={(value) => onOrganizationFilterChange(value === ALL ? null : value)}
        options={[
          { value: ALL, label: 'Alle Organisationen' },
          ...organizations.map((org) => ({ value: org.id, label: org.name })),
        ]}
      />

      <SelectField
        label="Rolle"
        hideLabel
        placeholder="Alle Rollen"
        value={roleFilter ?? ALL}
        onValueChange={(value) => onRoleFilterChange(value === ALL ? null : value)}
        options={[
          { value: ALL, label: 'Alle Rollen' },
          ...roles.map((role) => ({ value: role.id, label: role.name })),
        ]}
      />

      <p className="text-xs font-semibold uppercase tracking-wider text-dark-500">
        {memberCountLabel(members.length, total, filtered)}
      </p>

      {loading ? (
        <Loading className="justify-center py-6" label="Mitarbeitende werden geladen" />
      ) : members.length === 0 ? (
        <p className="text-sm text-dark-600">{emptyMessageOf(filtered)}</p>
      ) : (
        <ul role="list" aria-label="Mitarbeitende" className="flex list-none flex-col gap-2">
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
