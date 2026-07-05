import { UserRole } from '@green-ecolution/backend-client'
import { createEnumLookup, createEnumParser } from '@/lib/enumLookup'

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

export const getUserRoleDetails = createEnumLookup(UserRoleOptions)

export const parseUserRole = createEnumParser<UserRoleOrUnknown>(
  {
    tbz: UserRole.Tbz,
    'green-ecolution': UserRole.GreenEcolution,
    'smarte-grenzregion': UserRole.SmarteGrenzregion,
  },
  UNKNOWN_USER_ROLE,
)
