import { UserRole } from '@green-ecolution/backend-client'

export const UserRoleOptions = [
  {
    value: UserRole.Unknown,
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

export const getUserRoleDetails = (userRole: UserRole) =>
  UserRoleOptions.find((option) => option.value === userRole) ?? UserRoleOptions[0]

export const parseUserRole = (role: string): UserRole => {
  switch (role) {
    case 'tbz':
      return UserRole.Tbz
    case 'green-ecolution':
      return UserRole.GreenEcolution
    case 'smarte-grenzregion':
      return UserRole.SmarteGrenzregion
    default:
      return UserRole.Unknown
  }
}
