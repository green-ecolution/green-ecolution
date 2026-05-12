import { UserRole } from '@green-ecolution/backend-client'

// Local sentinel for roles not yet mapped to the backend's enum.
export const UNKNOWN_USER_ROLE = 'unknown' as const
export type UserRoleOrUnknown = UserRole | typeof UNKNOWN_USER_ROLE

export const UserRoleOptions: { value: UserRoleOrUnknown; label: string }[] = [
  {
    value: UNKNOWN_USER_ROLE,
    label: 'Keine Angabe',
  },
  {
    value: UserRole.GreenEcolution,
    label: 'Green Ecolution | HS Flensburg',
  },
  {
    value: UserRole.SmarteGrenzregion,
    label: 'Smarte Grenzregion',
  },
  {
    value: UserRole.Tbz,
    label: 'TBZ Flensburg',
  },
]

export const getUserRoleDetails = (userRole: UserRoleOrUnknown) =>
  UserRoleOptions.find((option) => option.value === userRole) ?? UserRoleOptions[0]

export const parseUserRole = (role: string): UserRoleOrUnknown => {
  switch (role) {
    case 'tbz':
      return UserRole.Tbz
    case 'green-ecolution':
      return UserRole.GreenEcolution
    case 'smarte-grenzregion':
      return UserRole.SmarteGrenzregion
    default:
      return UNKNOWN_USER_ROLE
  }
}
