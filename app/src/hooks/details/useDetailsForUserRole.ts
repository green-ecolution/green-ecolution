import { UserRole } from '@green-ecolution/backend-client'

export const UserRoleOptions = [
  {
    value: UserRole.UserRoleUnknown,
    label: 'Keine Angabe',
  },
  {
    value: UserRole.UserRoleGreenEcolution,
    label: 'Green Ecolution | HS Flensburg',
  },
  {
    value: UserRole.UserRoleSmarteGrenzregion,
    label: 'Smarte Grenzregion',
  },
  {
    value: UserRole.UserRoleTbz,
    label: 'TBZ Flensburg',
  },
]

export const getUserRoleDetails = (userRole: UserRole) =>
  UserRoleOptions.find((option) => option.value === userRole) ?? UserRoleOptions[0]

export const parseUserRole = (role: string): UserRole => {
  switch (role) {
    case 'tbz':
      return UserRole.UserRoleTbz
    case 'green-ecolution':
      return UserRole.UserRoleGreenEcolution
    case 'smarte-grenzregion':
      return UserRole.UserRoleSmarteGrenzregion
    default:
      return UserRole.UserRoleUnknown
  }
}
