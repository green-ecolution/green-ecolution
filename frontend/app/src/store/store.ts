import { create, StateCreator } from 'zustand'
import { devtools, persist } from 'zustand/middleware'
import { immer } from 'zustand/middleware/immer'
import { MAP_DEFAULT_CENTER, MAP_MIN_ZOOM } from '@/lib/mapConfig'
import { FormDraftSlice } from './form/formDraftSlice'
import { emptyFilters, FilterDraftSlice } from './filter/filterDraftSlice'

// Live map viewport, mirrored from MapLibre on every move/zoom end. On /map the
// URL search params are the deep-link source of truth (synced debounced in
// useMapStoreSync); embedded maps elsewhere read this as their initial position.
interface MapSlice {
  mapCenter: [number, number]
  mapZoom: number
  setMapCenter: (center: [number, number]) => void
  setMapZoom: (zoom: number) => void
}

interface SidebarSlice {
  sidebarCollapsed: boolean | null
  setSidebarCollapsed: (collapsed: boolean) => void
}

type Store = MapSlice & SidebarSlice & FormDraftSlice & FilterDraftSlice
type Mutators = [
  ['zustand/devtools', never],
  ['zustand/persist', unknown],
  ['zustand/immer', never],
]

const createMapSlice: StateCreator<Store, Mutators, [], MapSlice> = (set) => ({
  mapCenter: MAP_DEFAULT_CENTER,
  mapZoom: MAP_MIN_ZOOM,
  setMapCenter: (center) =>
    set((state) => {
      state.mapCenter = center
    }),
  setMapZoom: (zoom) =>
    set((state) => {
      state.mapZoom = zoom
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

const createFilterDraftSlice: StateCreator<Store, Mutators, [], FilterDraftSlice> = (set) => ({
  filterDraft: emptyFilters(),

  setFilterStatusTags: (value) =>
    set((state) => {
      state.filterDraft.statusTags = value
    }),

  setFilterRegionTags: (value) =>
    set((state) => {
      state.filterDraft.regionTags = value
    }),

  setFilterSoilTags: (value) =>
    set((state) => {
      state.filterDraft.soilTags = value
    }),

  setFilterHasCluster: (value) =>
    set((state) => {
      state.filterDraft.hasCluster = value
    }),

  setFilterPlantingYearRange: (range) =>
    set((state) => {
      if (range.length !== 2) return
      const [min, max] = range
      state.filterDraft.plantingYears = Array.from({ length: max - min + 1 }, (_, i) => min + i)
    }),

  seedFilterDraft: (filters) =>
    set((state) => {
      state.filterDraft = filters
    }),

  resetFilterDraft: () =>
    set((state) => {
      state.filterDraft = emptyFilters()
    }),
})

const useStore = create<Store>()(
  devtools(
    persist(
      immer((...a) => ({
        ...createMapSlice(...a),
        ...createSidebarSlice(...a),
        ...createFormDraftSlice(...a),
        ...createFilterDraftSlice(...a),
      })),
      {
        name: 'green-ecolution-preferences',
        partialize: (s) => ({ sidebarCollapsed: s.sidebarCollapsed }),
      },
    ),
  ),
)

export default useStore
