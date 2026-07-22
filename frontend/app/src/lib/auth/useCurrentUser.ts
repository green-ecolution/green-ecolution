import { useMemo } from 'react'
import { decodeJWT } from '@/lib/utils'
import { KeycloakJWT } from '@/lib/types/keycloak'
import {
  parseUserStatus,
  UNKNOWN_USER_STATUS,
  UserStatusOrUnknown,
} from '@/hooks/details/useDetailsForUserStatus'
import { parseDrivingLicense } from '@/hooks/details/useDetailsForDrivingLicense'
import { DrivingLicense } from '@green-ecolution/backend-client'
import { useAuthSession } from './authSessionContext'

export interface CurrentUser {
  username: string
  email: string
  firstName: string
  lastName: string
  drivingLicenses: DrivingLicense[]
  userStatus: UserStatusOrUnknown
}

const EMPTY: CurrentUser = {
  username: '',
  email: '',
  firstName: '',
  lastName: '',
  drivingLicenses: [],
  userStatus: UNKNOWN_USER_STATUS,
}

export function useCurrentUser(): CurrentUser {
  const { accessToken } = useAuthSession()
  return useMemo(() => {
    if (!accessToken) return EMPTY
    let jwt: KeycloakJWT
    try {
      jwt = decodeJWT<KeycloakJWT>(accessToken)
    } catch {
      return EMPTY
    }
    return {
      username: jwt.preferred_username,
      email: jwt.email,
      firstName: jwt.given_name,
      lastName: jwt.family_name,
      drivingLicenses: jwt.driving_licenses ? jwt.driving_licenses.map(parseDrivingLicense) : [],
      userStatus: jwt.status ? parseUserStatus(jwt.status) : UNKNOWN_USER_STATUS,
    }
  }, [accessToken])
}
