import { describe, it, expect } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import FilterProvider, { useFilter } from './FilterContext'
import { ReactNode } from 'react'

const wrapper = ({ children }: { children: ReactNode }) => (
  <FilterProvider>{children}</FilterProvider>
)

describe('FilterContext', () => {
  describe('handleClusterChange', () => {
    it('sets hasCluster to true when "Gruppe zugehörig" is checked', () => {
      const { result } = renderHook(() => useFilter(), { wrapper })

      act(() => {
        result.current.handleClusterChange({
          target: { checked: true, value: 'true' },
        } as React.ChangeEvent<HTMLInputElement>)
      })

      expect(result.current.filters.hasCluster).toBe(true)
    })

    it('sets hasCluster to false when "Keiner Gruppe zugehörig" is checked', () => {
      const { result } = renderHook(() => useFilter(), { wrapper })

      act(() => {
        result.current.handleClusterChange({
          target: { checked: true, value: 'false' },
        } as React.ChangeEvent<HTMLInputElement>)
      })

      expect(result.current.filters.hasCluster).toBe(false)
    })

    it('resets hasCluster to undefined when option is unchecked', () => {
      const { result } = renderHook(() => useFilter(), { wrapper })

      // First select an option
      act(() => {
        result.current.handleClusterChange({
          target: { checked: true, value: 'true' },
        } as React.ChangeEvent<HTMLInputElement>)
      })
      expect(result.current.filters.hasCluster).toBe(true)

      // Then uncheck it - this should reset to undefined
      act(() => {
        result.current.handleClusterChange({
          target: { checked: false, value: 'true' },
        } as React.ChangeEvent<HTMLInputElement>)
      })
      expect(result.current.filters.hasCluster).toBeUndefined()
    })
  })

  describe('handleStatusChange', () => {
    it('adds status to array when checked', () => {
      const { result } = renderHook(() => useFilter(), { wrapper })

      act(() => {
        result.current.handleStatusChange({
          target: { checked: true, value: 'good' },
        } as React.ChangeEvent<HTMLInputElement>)
      })

      expect(result.current.filters.statusTags).toContain('good')
    })

    it('removes status from array when unchecked', () => {
      const { result } = renderHook(() => useFilter(), { wrapper })

      act(() => {
        result.current.handleStatusChange({
          target: { checked: true, value: 'good' },
        } as React.ChangeEvent<HTMLInputElement>)
      })

      act(() => {
        result.current.handleStatusChange({
          target: { checked: false, value: 'good' },
        } as React.ChangeEvent<HTMLInputElement>)
      })

      expect(result.current.filters.statusTags).not.toContain('good')
    })
  })

  describe('handlePlantingYearChange', () => {
    it('adds year to array when checked', () => {
      const { result } = renderHook(() => useFilter(), { wrapper })

      act(() => {
        result.current.handlePlantingYearChange({
          target: { checked: true, value: '2024' },
        } as React.ChangeEvent<HTMLInputElement>)
      })

      expect(result.current.filters.plantingYears).toContain(2024)
    })

    it('removes year from array when unchecked', () => {
      const { result } = renderHook(() => useFilter(), { wrapper })

      act(() => {
        result.current.handlePlantingYearChange({
          target: { checked: true, value: '2024' },
        } as React.ChangeEvent<HTMLInputElement>)
      })

      act(() => {
        result.current.handlePlantingYearChange({
          target: { checked: false, value: '2024' },
        } as React.ChangeEvent<HTMLInputElement>)
      })

      expect(result.current.filters.plantingYears).not.toContain(2024)
    })
  })

  describe('resetFilters', () => {
    it('resets all filters to initial state', () => {
      const { result } = renderHook(() => useFilter(), { wrapper })

      // Set some filters
      act(() => {
        result.current.handleClusterChange({
          target: { checked: true, value: 'true' },
        } as React.ChangeEvent<HTMLInputElement>)
        result.current.handleStatusChange({
          target: { checked: true, value: 'good' },
        } as React.ChangeEvent<HTMLInputElement>)
        result.current.handlePlantingYearChange({
          target: { checked: true, value: '2024' },
        } as React.ChangeEvent<HTMLInputElement>)
      })

      // Reset
      act(() => {
        result.current.resetFilters()
      })

      expect(result.current.filters.hasCluster).toBeUndefined()
      expect(result.current.filters.statusTags).toEqual([])
      expect(result.current.filters.plantingYears).toEqual([])
      expect(result.current.filters.regionTags).toEqual([])
    })
  })

  describe('applyOldStateToTags', () => {
    it('restores previous filter state', () => {
      const { result } = renderHook(() => useFilter(), { wrapper })

      const oldState = {
        statusTags: ['good'],
        regionTags: ['region1'],
        hasCluster: true,
        plantingYears: [2024],
      }

      act(() => {
        result.current.applyOldStateToTags(oldState)
      })

      expect(result.current.filters.statusTags).toEqual(['good'])
      expect(result.current.filters.regionTags).toEqual(['region1'])
      expect(result.current.filters.hasCluster).toBe(true)
      expect(result.current.filters.plantingYears).toEqual([2024])
    })
  })
})
