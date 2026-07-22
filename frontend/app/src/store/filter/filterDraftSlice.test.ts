import { describe, it, expect, beforeEach } from 'vitest'
import useStore from '../store'
import { emptyFilters } from './filterDraftSlice'

describe('filterDraftSlice', () => {
  beforeEach(() => {
    useStore.setState({ filterDraft: emptyFilters() })
  })

  describe('setFilterHasCluster', () => {
    it('sets hasCluster to true and false', () => {
      useStore.getState().setFilterHasCluster(true)
      expect(useStore.getState().filterDraft.hasCluster).toBe(true)

      useStore.getState().setFilterHasCluster(false)
      expect(useStore.getState().filterDraft.hasCluster).toBe(false)
    })

    it('resets hasCluster to undefined', () => {
      useStore.getState().setFilterHasCluster(true)
      useStore.getState().setFilterHasCluster(undefined)
      expect(useStore.getState().filterDraft.hasCluster).toBeUndefined()
    })
  })

  describe('setFilterPlantingYearRange', () => {
    it('expands a range to the full list of years', () => {
      useStore.getState().setFilterPlantingYearRange([2020, 2023])
      expect(useStore.getState().filterDraft.plantingYears).toEqual([2020, 2021, 2022, 2023])
    })

    it('ignores values that are not a [min, max] pair', () => {
      useStore.getState().setFilterPlantingYearRange([2020])
      expect(useStore.getState().filterDraft.plantingYears).toEqual([])
    })
  })

  describe('tag setters', () => {
    it('replace their tag list', () => {
      useStore.getState().setFilterStatusTags(['good'])
      useStore.getState().setFilterRegionTags(['region1'])
      useStore.getState().setFilterSoilTags(['sandy'])

      const { filterDraft } = useStore.getState()
      expect(filterDraft.statusTags).toEqual(['good'])
      expect(filterDraft.regionTags).toEqual(['region1'])
      expect(filterDraft.soilTags).toEqual(['sandy'])
    })
  })

  describe('resetFilterDraft', () => {
    it('resets all filters to the empty state', () => {
      useStore.getState().setFilterHasCluster(true)
      useStore.getState().setFilterStatusTags(['good'])
      useStore.getState().setFilterPlantingYearRange([2024, 2024])

      useStore.getState().resetFilterDraft()

      expect(useStore.getState().filterDraft).toEqual(emptyFilters())
    })
  })

  describe('seedFilterDraft', () => {
    it('replaces the draft with the applied filters', () => {
      const applied = {
        statusTags: ['good'],
        regionTags: ['region1'],
        soilTags: [],
        hasCluster: true,
        plantingYears: [2024],
      }

      useStore.getState().seedFilterDraft(applied)

      expect(useStore.getState().filterDraft).toEqual(applied)
    })
  })

  it('is not persisted to localStorage', () => {
    useStore.getState().setFilterStatusTags(['good'])

    const raw = localStorage.getItem('green-ecolution-preferences')
    expect(raw).not.toBeNull()

    const persisted = JSON.parse(raw!) as { state: Record<string, unknown> }
    expect(persisted.state).not.toHaveProperty('filterDraft')
  })
})
