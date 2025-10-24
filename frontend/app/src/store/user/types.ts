import { DrivingLicense, UserRole, UserStatus } from '@green-ecolution/backend-client'

interface UserState {
  username: string
  email: string
  firstName: string
  lastName: string
  drivingLicenses: DrivingLicense[]
  userRoles: UserRole[]
  status: UserStatus
}

interface UserActions {
  setUsername: (username: string) => void
  setEmail: (email: string) => void
  setFirstName: (firstName: string) => void
  setLastName: (lastName: string) => void
  setDrivingLicenses: (drivingLicenses: DrivingLicense[]) => void
  setUserRoles: (userRoles: UserRole[]) => void
  setStatus: (status: UserStatus) => void
  setFromJwt: (jwt: string) => void
  isEmpty: () => boolean
  clear: () => void
}

export type UserStore = UserState & UserActions
