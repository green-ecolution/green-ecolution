import { create, StateCreator } from 'zustand'
import { devtools, persist } from 'zustand/middleware'
import { immer } from 'zustand/middleware/immer'
import { useShallow } from 'zustand/react/shallow'
import { FormDraftSlice } from './form/formDraftSlice'

interface MapSlice {
  mapCenter: [number, number]
  mapZoom: number
  mapMinZoom: number
  mapMaxZoom: number
  mapSearchTerm: string
  setMapCenter: (center: [number, number]) => void
  setMapZoom: (zoom: number) => void
  setMapSearchTerm: (term: string) => void
}

interface SidebarSlice {
  sidebarCollapsed: boolean | null
  setSidebarCollapsed: (collapsed: boolean) => void
}

type Store = MapSlice & SidebarSlice & FormDraftSlice
type Mutators = [
  ['zustand/devtools', never],
  ['zustand/persist', unknown],
  ['zustand/immer', never],
]

const createMapSlice: StateCreator<Store, Mutators, [], MapSlice> = (set) => ({
  mapCenter: [54.792277136221905, 9.43580607453268],
  mapZoom: 13,
  mapMinZoom: 13,
  mapMaxZoom: 18,
  mapSearchTerm: '',
  setMapCenter: (center) =>
    set((state) => {
      state.mapCenter = center
    }),
  setMapZoom: (zoom) =>
    set((state) => {
      state.mapZoom = zoom
    }),
  setMapSearchTerm: (term) =>
    set((state) => {
      state.mapSearchTerm = term
    }),
})

const createSidebarSlice: StateCreator<Store, Mutators, [], SidebarSlice> = (set) => ({
  sidebarCollapsed: null,
  setSidebarCollapsed: (collapsed) =>
    set((state) => {
      state.sidebarCollapsed = collapsed
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

const useStore = create<Store>()(
  devtools(
    persist(
      immer((...a) => ({
        ...createMapSlice(...a),
        ...createSidebarSlice(...a),
        ...createFormDraftSlice(...a),
      })),
      {
        name: 'green-ecolution-preferences',
        partialize: (s) => ({ sidebarCollapsed: s.sidebarCollapsed }),
      },
    ),
  ),
)

const mapSelector = (s: Store) => ({
  mapCenter: s.mapCenter,
  mapZoom: s.mapZoom,
  mapMinZoom: s.mapMinZoom,
  mapMaxZoom: s.mapMaxZoom,
  setMapCenter: s.setMapCenter,
  setMapZoom: s.setMapZoom,
})

export const useMapStore = () => useStore(useShallow(mapSelector))

export default useStore
