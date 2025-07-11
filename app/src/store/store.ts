import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import { immer } from 'zustand/middleware/immer'
import { AuthStore } from './auth/types'
import { UserStore } from './user/types'
import { WritableDraft } from 'immer'
import { userStore } from './user/userStore'
import { authStore } from './auth/authStore'
import { MapStore } from './map/types'
import { mapStore } from './map/mapStore'

export interface Store {
  auth: AuthStore
  user: UserStore
  map: MapStore
}

export type SubStore<T> = (
  set: (
    nextStateOrUpdater: Store | Partial<Store> | ((state: WritableDraft<Store>) => void),
    shouldReplace?: boolean,
  ) => void,
  get: () => Store,
) => T

const useStore = create<Store>()(
  devtools(
    immer((set, get) => ({
      auth: authStore(set, get),
      user: userStore(set, get),
      map: mapStore(set, get),
    })),
  ),
)

export const useAuthStore = () => useStore((state) => state.auth)
export const useUserStore = () => useStore((state) => state.user)
export const useMapStore = () => useStore((state) => state.map)

export default useStore
