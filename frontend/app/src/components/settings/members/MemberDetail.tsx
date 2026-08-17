import { Badge } from '@green-ecolution/ui'
import type { DrivingLicense, UserStatus } from '@green-ecolution/backend-client'
import type { OrganizationResponse, RoleResponse, UserResponse } from '@/api/backendApi'
import { getUserStatusDetails } from '@/hooks/details/useDetailsForUserStatus'
import { initialsOf } from '@/lib/initials'
import { TILE } from './cardChrome'
import MemberActionButtons from './MemberActionButtons'
import { fullNameOf, sinceLabel } from './memberList'
import MemberOrganizationCard from './MemberOrganizationCard'
import MemberProfileCard from './MemberProfileCard'
import MemberRolesCard from './MemberRolesCard'
import type { MemberProfileDraft } from './useMemberProfileDraft'

interface MemberDetailProps {
  user: UserResponse
  draft: MemberProfileDraft
  dirty: boolean
  isSelf: boolean
  canUpdate: boolean
  canReadRoles: boolean
  canReadOrganizations: boolean
  organizations: OrganizationResponse[]
  assignableRoles: RoleResponse[]
  roleError: string | null
  organizationError: string | null
  saving: boolean
  onStatusChange: (status: UserStatus) => void
  onDrivingLicensesChange: (licenses: DrivingLicense[]) => void
  onPhoneNumberChange: (value: string) => void
  onEmployeeIdChange: (value: string) => void
  onRoleAssign: (roleId: string) => void
  onRoleRevoke: (roleId: string) => void
  onOrganizationChange: (orgId: string) => void
  onSave: () => void
  onCancel: () => void
  renderActionBar: boolean
}

const SELF_ORGANIZATION_LOCKED =
  'Die eigene Organisation kann hier nicht geändert werden. Das muss eine andere Person mit Verwaltungsrechten übernehmen.'
const SELF_ROLES_LOCKED =
  'Die eigenen Rollen können hier nicht geändert werden. Das muss eine andere Person mit Verwaltungsrechten übernehmen.'
const NO_ORGANIZATION_ROLES_LOCKED =
  'Ohne Organisation lassen sich keine Rollen zuweisen. Ordne die Person zuerst einer Organisation zu.'

const MemberDetail = ({
  user,
  draft,
  dirty,
  isSelf,
  canUpdate,
  canReadRoles,
  canReadOrganizations,
  organizations,
  assignableRoles,
  roleError,
  organizationError,
  saving,
  onStatusChange,
  onDrivingLicensesChange,
  onPhoneNumberChange,
  onEmployeeIdChange,
  onRoleAssign,
  onRoleRevoke,
  onOrganizationChange,
  onSave,
  onCancel,
  renderActionBar,
}: MemberDetailProps) => {
  const status = getUserStatusDetails(user.status)
  const since = sinceLabel(user.createdAt)
  const organization = user.organization ?? undefined

  const organizationLockedReason = isSelf ? SELF_ORGANIZATION_LOCKED : null
  const rolesLockedReason = isSelf
    ? SELF_ROLES_LOCKED
    : organization === undefined
      ? NO_ORGANIZATION_ROLES_LOCKED
      : null

  return (
    <div className="flex flex-col gap-6">
      <header className="flex min-w-0 items-start gap-4">
        <span className={`${TILE} size-12 rounded-xl bg-green-dark text-base text-white`}>
          {initialsOf(user.firstName, user.lastName)}
        </span>
        <div className="min-w-0 flex-1">
          <h2 className="font-lato text-2xl font-bold text-dark break-words">{fullNameOf(user)}</h2>
          <p className="mt-0.5 flex flex-wrap items-center gap-x-1.5 gap-y-1 text-sm text-dark-600">
            <span className="truncate">
              {user.username} · {user.email}
              {since ? ` · seit ${since}` : ''}
            </span>
            {!user.emailVerified && <Badge variant="outline-yellow">E-Mail nicht bestätigt</Badge>}
          </p>
        </div>
        <Badge variant={status.color}>{status.label}</Badge>
      </header>

      <div className="@container">
        <div className="grid gap-4 @min-[48rem]:grid-cols-2">
          <MemberOrganizationCard
            organization={organization}
            organizations={organizations}
            editable={canUpdate && canReadOrganizations && !isSelf}
            lockedReason={organizationLockedReason}
            error={organizationError}
            onChange={onOrganizationChange}
          />

          <MemberRolesCard
            roles={user.roles}
            assignable={assignableRoles}
            editable={canUpdate && canReadRoles && !isSelf && organization !== undefined}
            lockedReason={rolesLockedReason}
            error={roleError}
            onAssign={onRoleAssign}
            onRevoke={onRoleRevoke}
          />

          <MemberProfileCard
            draft={draft}
            editable={canUpdate}
            onStatusChange={onStatusChange}
            onDrivingLicensesChange={onDrivingLicensesChange}
            onPhoneNumberChange={onPhoneNumberChange}
            onEmployeeIdChange={onEmployeeIdChange}
          />
        </div>
      </div>

      {renderActionBar && dirty && canUpdate && (
        <div className="sticky bottom-0 -mt-6 flex flex-col-reverse gap-2 border-t border-dark-200 bg-[var(--member-panel-bg,var(--color-dark-50))] pb-3 pt-6 sm:flex-row sm:items-center sm:justify-end sm:gap-3">
          <MemberActionButtons saving={saving} onSave={onSave} onCancel={onCancel} />
        </div>
      )}
    </div>
  )
}

export default MemberDetail
