import { decodeJWT } from '@/lib/utils'
import { SubStore } from '../store'
import { UserStore } from './types'
import { KeycloakJWT } from '@/lib/types/keycloak'
import { parseUserRole } from '@/hooks/details/useDetailsForUserRole'
import { parseUserStatus } from '@/hooks/details/useDetailsForUserStatus'
import { UserStatus } from '@green-ecolution/backend-client'
import { parseDrivingLicense } from '@/hooks/details/useDetailsForDrivingLicense'

export const userStore: SubStore<UserStore> = (set, get) => ({
  username: '',
  email: '',
  firstName: '',
  lastName: '',
  drivingLicenses: [],
  userRoles: [],
  status: UserStatus.UserStatusUnknown,
  setUsername: (username) =>
    set((state) => {
      state.user.username = username
    }),
  setEmail: (email) =>
    set((state) => {
      state.user.email = email
    }),
  setFirstName: (firstName) =>
    set((state) => {
      state.user.firstName = firstName
    }),
  setLastName: (lastName) =>
    set((state) => {
      state.user.lastName = lastName
    }),
  setDrivingLicenses: (drivingLicenses) =>
    set((state) => {
      state.user.drivingLicenses = drivingLicenses
    }),
  setStatus: (status) =>
    set((state) => {
      state.user.status = status
    }),
  setUserRoles: (userRoles) =>
    set((state) => {
      state.user.userRoles = userRoles
    }),
  setFromJwt: (jwt) =>
    set((state) => {
      const jwtInfo = decodeJWT<KeycloakJWT>(jwt)
      if (jwtInfo) {
        state.user.email = jwtInfo.email
        state.user.username = jwtInfo.preferred_username
        state.user.firstName = jwtInfo.given_name
        state.user.lastName = jwtInfo.family_name
        state.user.drivingLicenses = jwtInfo.driving_licenses
          ? jwtInfo.driving_licenses.map(parseDrivingLicense)
          : []
        state.user.userRoles = jwtInfo.user_roles ? jwtInfo.user_roles.map(parseUserRole) : []
        state.user.status = parseUserStatus(jwtInfo.status)
      }
    }),
  isEmpty: () => {
    return (
      !get().user.username ||
      !get().user.email ||
      !get().user.firstName ||
      !get().user.lastName ||
      !get().user.drivingLicenses ||
      !get().user.userRoles ||
      !get().user.status
    )
  },
  clear: () =>
    set((state) => {
      state.user.username = ''
      state.user.email = ''
      state.user.firstName = ''
      state.user.lastName = ''
      state.user.drivingLicenses = []
      state.user.userRoles = []
      state.user.status = UserStatus.UserStatusUnknown
    }),
})
