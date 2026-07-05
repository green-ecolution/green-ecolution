import { UserStatus } from '@green-ecolution/backend-client'
import { createEnumLookup, createEnumParser } from '@/lib/enumLookup'
import { StatusColor } from './types'

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

export const getUserStatusDetails = createEnumLookup(UserStatusOptions)

const parseLowercaseUserStatus = createEnumParser<UserStatusOrUnknown>(
  {
    absent: UserStatus.Absent,
    available: UserStatus.Available,
  },
  UNKNOWN_USER_STATUS,
)

export const parseUserStatus = (status: string): UserStatusOrUnknown =>
  parseLowercaseUserStatus(status.toLowerCase())
