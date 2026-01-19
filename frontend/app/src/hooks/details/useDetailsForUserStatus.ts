import { UserStatus } from '@green-ecolution/backend-client'
import { StatusColor } from './useDetailsForWateringPlanStatus'

export const UserStatusOptions: {
  value: UserStatus
  label: string
  color: StatusColor
}[] = [
  {
    value: UserStatus.UserStatusUnknown,
    label: 'Unbekannt',
    color: 'outline-dark',
  },
  {
    value: UserStatus.UserStatusAbsent,
    label: 'Nicht verfügbar',
    color: 'outline-red',
  },
  {
    value: UserStatus.UserStatusAvailable,
    label: 'Verfügbar',
    color: 'outline-green-dark',
  },
]

export const getUserStatusDetails = (userStatus: UserStatus) =>
  UserStatusOptions.find((option) => option.value === userStatus) ?? UserStatusOptions[0]

export const parseUserStatus = (status: string): UserStatus => {
  switch (status.toLowerCase()) {
    case 'absent':
      return UserStatus.UserStatusAbsent
    case 'available':
      return UserStatus.UserStatusAvailable
    default:
      return UserStatus.UserStatusUnknown
  }
}
