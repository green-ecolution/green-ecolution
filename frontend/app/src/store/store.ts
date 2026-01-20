import { create, StateCreator } from 'zustand'
import { devtools } from 'zustand/middleware'
import { immer } from 'zustand/middleware/immer'
import { useShallow } from 'zustand/react/shallow'
import { ClientToken, DrivingLicense, UserRole, UserStatus } from '@green-ecolution/backend-client'
import { decodeJWT } from '@/lib/utils'
import { KeycloakJWT } from '@/lib/types/keycloak'
import { parseUserRole } from '@/hooks/details/useDetailsForUserRole'
import { parseUserStatus } from '@/hooks/details/useDetailsForUserStatus'
import { parseDrivingLicense } from '@/hooks/details/useDetailsForDrivingLicense'
import { FormDraftSlice } from './form/formDraftSlice'

// =============================================================================
// Types
// =============================================================================

interface AuthSlice {
  isAuthenticated: boolean
  token: ClientToken | null
  setToken: (token: ClientToken) => void
  clearAuth: () => void
}

interface UserSlice {
  username: string
  email: string
  firstName: string
  lastName: string
  drivingLicenses: DrivingLicense[]
  userRoles: UserRole[]
  userStatus: UserStatus
  setUserFromJwt: (jwt: string) => void
  isUserEmpty: () => boolean
  clearUser: () => void
}

interface MapSlice {
  mapCenter: [number, number]
  mapZoom: number
  mapMinZoom: number
  mapMaxZoom: number
  showSelectModal: boolean
  setMapCenter: (center: [number, number]) => void
  setMapZoom: (zoom: number) => void
  setShowSelectModal: (show: boolean) => void
}

type Store = AuthSlice & UserSlice & MapSlice & FormDraftSlice
type Mutators = [['zustand/devtools', never], ['zustand/immer', never]]

// =============================================================================
// Slices
// =============================================================================

const createAuthSlice: StateCreator<Store, Mutators, [], AuthSlice> = (set) => ({
  isAuthenticated: !!localStorage.getItem('refreshToken'),
  token: null,
  setToken: (token) =>
    set((state) => {
      localStorage.setItem('refreshToken', token.refreshToken)
      state.isAuthenticated = true
      state.token = token
    }),
  clearAuth: () =>
    set((state) => {
      localStorage.removeItem('refreshToken')
      state.isAuthenticated = false
      state.token = null
    }),
})

const createUserSlice: StateCreator<Store, Mutators, [], UserSlice> = (set, get) => ({
  username: '',
  email: '',
  firstName: '',
  lastName: '',
  drivingLicenses: [],
  userRoles: [],
  userStatus: UserStatus.UserStatusUnknown,
  setUserFromJwt: (jwt) =>
    set((state) => {
      const jwtInfo = decodeJWT<KeycloakJWT>(jwt)
      if (jwtInfo) {
        state.username = jwtInfo.preferred_username
        state.email = jwtInfo.email
        state.firstName = jwtInfo.given_name
        state.lastName = jwtInfo.family_name
        state.drivingLicenses = jwtInfo.driving_licenses
          ? jwtInfo.driving_licenses.map(parseDrivingLicense)
          : []
        state.userRoles = jwtInfo.user_roles ? jwtInfo.user_roles.map(parseUserRole) : []
        state.userStatus = parseUserStatus(jwtInfo.status)
      }
    }),
  isUserEmpty: () => {
    const s = get()
    return (
      !s.username ||
      !s.email ||
      !s.firstName ||
      !s.lastName ||
      s.drivingLicenses.length === 0 ||
      s.userRoles.length === 0 ||
      s.userStatus === UserStatus.UserStatusUnknown
    )
  },
  clearUser: () =>
    set((state) => {
      state.username = ''
      state.email = ''
      state.firstName = ''
      state.lastName = ''
      state.drivingLicenses = []
      state.userRoles = []
      state.userStatus = UserStatus.UserStatusUnknown
    }),
})

const createMapSlice: StateCreator<Store, Mutators, [], MapSlice> = (set) => ({
  mapCenter: [54.792277136221905, 9.43580607453268],
  mapZoom: 13,
  mapMinZoom: 13,
  mapMaxZoom: 18,
  showSelectModal: false,
  setMapCenter: (center) =>
    set((state) => {
      state.mapCenter = center
    }),
  setMapZoom: (zoom) =>
    set((state) => {
      state.mapZoom = zoom
    }),
  setShowSelectModal: (show) =>
    set((state) => {
      state.showSelectModal = show
    }),
})

const createFormDraftSlice: StateCreator<Store, Mutators, [], FormDraftSlice> = (set) => ({
  formDrafts: {},

  setFormDraft: (key, data) =>
    set((state) => {
      state.formDrafts[key] = { data, hasChanges: false }
    }),

  updateFormDraft: (key, updater) =>
    set((state) => {
      const current = (state.formDrafts[key]?.data as Parameters<typeof updater>[0]) ?? null
      state.formDrafts[key] = { data: updater(current), hasChanges: true }
    }),

  markFormDraftChanged: (key) =>
    set((state) => {
      if (state.formDrafts[key]) {
        state.formDrafts[key].hasChanges = true
      } else {
        state.formDrafts[key] = { data: null, hasChanges: true }
      }
    }),

  clearFormDraft: (key) =>
    set((state) => {
      delete state.formDrafts[key]
    }),

  clearAllFormDrafts: () =>
    set((state) => {
      state.formDrafts = {}
    }),
})

// =============================================================================
// Store
// =============================================================================

const useStore = create<Store>()(
  devtools(
    immer((...a) => ({
      ...createAuthSlice(...a),
      ...createUserSlice(...a),
      ...createMapSlice(...a),
      ...createFormDraftSlice(...a),
    })),
  ),
)

// =============================================================================
// Selectors (defined outside hooks for stable references)
// =============================================================================

const authSelector = (s: Store) => ({
  isAuthenticated: s.isAuthenticated,
  token: s.token,
  setToken: s.setToken,
  clearAuth: s.clearAuth,
})

const userSelector = (s: Store) => ({
  username: s.username,
  email: s.email,
  firstName: s.firstName,
  lastName: s.lastName,
  drivingLicenses: s.drivingLicenses,
  userRoles: s.userRoles,
  userStatus: s.userStatus,
  setUserFromJwt: s.setUserFromJwt,
  isUserEmpty: s.isUserEmpty,
  clearUser: s.clearUser,
})

const mapSelector = (s: Store) => ({
  mapCenter: s.mapCenter,
  mapZoom: s.mapZoom,
  mapMinZoom: s.mapMinZoom,
  mapMaxZoom: s.mapMaxZoom,
  showSelectModal: s.showSelectModal,
  setMapCenter: s.setMapCenter,
  setMapZoom: s.setMapZoom,
  setShowSelectModal: s.setShowSelectModal,
})

// =============================================================================
// Selector Hooks
// =============================================================================

export const useAuthStore = () => useStore(useShallow(authSelector))
export const useUserStore = () => useStore(useShallow(userSelector))
export const useMapStore = () => useStore(useShallow(mapSelector))

export default useStore
