import { useCallback, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { UserStatus } from '@green-ecolution/backend-client'
import { createEnumParser } from '@/lib/enumLookup'
import { EnumsTranslate, StatusColor } from './types'

// Local sentinel for statuses outside the backend enum (e.g. unparseable JWT claims).
export const UNKNOWN_USER_STATUS = 'unknown' as const
export type UserStatusOrUnknown = UserStatus | typeof UNKNOWN_USER_STATUS

const UserStatusEntries: { value: UserStatusOrUnknown; color: StatusColor }[] = [
  { value: UNKNOWN_USER_STATUS, color: 'outline-dark' },
  { value: UserStatus.Absent, color: 'outline-red' },
  { value: UserStatus.Available, color: 'outline-green-dark' },
]

export interface UserStatusDetails {
  value: UserStatusOrUnknown
  color: StatusColor
  label: string
}

/** Reactive to language change: re-renders whichever component calls it. */
export const useUserStatusDetails = (): ((status: UserStatusOrUnknown) => UserStatusDetails) => {
  const { t } = useTranslation('enums')
  const translate = t as EnumsTranslate
  return useCallback(
    (status: UserStatusOrUnknown): UserStatusDetails => {
      const entry =
        UserStatusEntries.find((option) => option.value === status) ?? UserStatusEntries[0]
      return { ...entry, label: translate(`userStatus.${entry.value}.label`) }
    },
    [translate],
  )
}

/** The full status list with translated labels, in display order. */
export const useUserStatusOptions = (): UserStatusDetails[] => {
  const getDetails = useUserStatusDetails()
  return useMemo(() => UserStatusEntries.map((entry) => getDetails(entry.value)), [getDetails])
}

const parseLowercaseUserStatus = createEnumParser<UserStatusOrUnknown>(
  {
    absent: UserStatus.Absent,
    available: UserStatus.Available,
  },
  UNKNOWN_USER_STATUS,
)

export const parseUserStatus = (status: string): UserStatusOrUnknown =>
  parseLowercaseUserStatus(status.toLowerCase())
