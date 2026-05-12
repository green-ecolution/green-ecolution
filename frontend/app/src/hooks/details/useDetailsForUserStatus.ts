import { UserStatus } from '@green-ecolution/backend-client'
import { StatusColor } from './useDetailsForWateringPlanStatus'

// Local sentinel for statuses outside the backend enum (e.g. unparseable JWT claims).
export const UNKNOWN_USER_STATUS = 'unknown' as const
export type UserStatusOrUnknown = UserStatus | typeof UNKNOWN_USER_STATUS

export const UserStatusOptions: {
  value: UserStatusOrUnknown
  label: string
  color: StatusColor
}[] = [
  {
    value: UNKNOWN_USER_STATUS,
    label: 'Unbekannt',
    color: 'outline-dark',
  },
  {
    value: UserStatus.Absent,
    label: 'Nicht verfügbar',
    color: 'outline-red',
  },
  {
    value: UserStatus.Available,
    label: 'Verfügbar',
    color: 'outline-green-dark',
  },
]

export const getUserStatusDetails = (userStatus: UserStatusOrUnknown) =>
  UserStatusOptions.find((option) => option.value === userStatus) ?? UserStatusOptions[0]

export const parseUserStatus = (status: string): UserStatusOrUnknown => {
  switch (status.toLowerCase()) {
    case 'absent':
      return UserStatus.Absent
    case 'available':
      return UserStatus.Available
    default:
      return UNKNOWN_USER_STATUS
  }
}
