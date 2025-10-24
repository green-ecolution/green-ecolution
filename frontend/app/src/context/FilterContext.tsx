import React, { createContext, useState, ReactNode, use, useMemo } from 'react'

export interface Filters {
  statusTags: string[]
  regionTags: string[]
  hasCluster: boolean | undefined
  plantingYears: number[]
}

interface FilterContextType {
  filters: Filters
  handleStatusChange: (event: React.ChangeEvent<HTMLInputElement>) => void
  handleRegionChange: (event: React.ChangeEvent<HTMLInputElement>) => void
  handleClusterChange: (event: React.ChangeEvent<HTMLInputElement>) => void
  handlePlantingYearChange: (event: React.ChangeEvent<HTMLInputElement>) => void
  resetFilters: () => void
  applyOldStateToTags: (oldValues: Filters) => void
}

/* eslint-disable-next-line react-refresh/only-export-components */
export const FilterContext = createContext<FilterContextType | undefined>(undefined)

interface FilterProviderProps {
  initialStatus?: string[]
  initialRegions?: string[]
  initialHasCluster?: boolean | undefined
  initialPlantingYears?: number[]
  children: ReactNode
}

const defaultInitStatus: string[] = []
const defaultInitRegions: string[] = []
const defaultInitPlantingYears: number[] = []

const FilterProvider: React.FC<FilterProviderProps> = ({
  initialStatus = defaultInitStatus,
  initialRegions = defaultInitRegions,
  initialHasCluster = undefined,
  initialPlantingYears = defaultInitPlantingYears,
  children,
}) => {
  const [statusTags, setStatusTags] = useState(initialStatus)
  const [regionTags, setRegionTags] = useState(initialRegions)
  const [plantingYears, setPlantingYears] = useState(initialPlantingYears)
  const [hasCluster, setHasCluster] = useState(initialHasCluster)

  const handleStatusChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const { checked, value } = event.target
    setStatusTags((prev) => (checked ? [...prev, value] : prev.filter((tag) => tag !== value)))
  }

  const handleRegionChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const { checked, value } = event.target
    setRegionTags((prev) => (checked ? [...prev, value] : prev.filter((tag) => tag !== value)))
  }

  const handleClusterChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const { value } = event.target
    setHasCluster(value === 'true')
  }

  const handlePlantingYearChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const { checked, value } = event.target
    setPlantingYears((prev) =>
      checked ? [...prev, Number(value)] : prev.filter((year) => year !== Number(value)),
    )
  }

  const applyOldStateToTags = (oldValues: Filters) => {
    setStatusTags(oldValues.statusTags)
    setRegionTags(oldValues.regionTags)
    setHasCluster(oldValues.hasCluster)
    setPlantingYears(oldValues.plantingYears)
  }

  const resetFilters = () => {
    setStatusTags([])
    setRegionTags([])
    setHasCluster(undefined)
    setPlantingYears([])
  }

  const context = useMemo(
    () => ({
      filters: { statusTags, regionTags, hasCluster, plantingYears },
      handleStatusChange,
      handleRegionChange,
      handleClusterChange,
      handlePlantingYearChange,
      resetFilters,
      applyOldStateToTags,
    }),
    [hasCluster, plantingYears, regionTags, statusTags],
  )

  return <FilterContext value={context}>{children}</FilterContext>
}

/* eslint-disable-next-line react-refresh/only-export-components */
export const useFilter = () => {
  const context = use(FilterContext)
  if (context === undefined) {
    throw new Error('useFilter must be used within a FilterProvider')
  }
  return context
}

export default FilterProvider
