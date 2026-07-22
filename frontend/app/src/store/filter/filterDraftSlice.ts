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
