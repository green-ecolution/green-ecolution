import { create } from 'zustand'
import { devtools } from 'zustand/middleware'
import { immer } from 'zustand/middleware/immer'

export interface Filters {
  statusTags: string[]
  regionTags: string[]
  soilTags: string[]
  hasCluster: boolean | undefined
  plantingYears: number[]
}

export const emptyFilters = (): Filters => ({
  statusTags: [],
  regionTags: [],
  soilTags: [],
  hasCluster: undefined,
  plantingYears: [],
})

// Staging state for the filter dialog: seeded from the URL search params on
// open, written back to the URL on submit. The URL stays the source of truth
// for the applied filters.
export interface FilterDraftSlice {
  filterDraft: Filters
  setFilterStatusTags: (value: string[]) => void
  setFilterRegionTags: (value: string[]) => void
  setFilterSoilTags: (value: string[]) => void
  setFilterHasCluster: (value: boolean | undefined) => void
  setFilterPlantingYearRange: (range: number[]) => void
  seedFilterDraft: (filters: Filters) => void
  resetFilterDraft: () => void
}

const useIrrigationFilterStore = create<FilterDraftSlice>()(
  devtools(
    immer((set) => ({
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
    })),
  ),
)

export default useIrrigationFilterStore
