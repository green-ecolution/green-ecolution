import React, { createContext, useState, ReactNode, use, useMemo } from 'react'

export interface Filters {
  statusTags: string[]
  regionTags: string[]
  soilTags: string[]
  hasCluster: boolean | undefined
  plantingYears: number[]
}

interface FilterContextType {
  filters: Filters
  handleStatusChange: (event: React.ChangeEvent<HTMLInputElement>) => void
  handleRegionChange: (event: React.ChangeEvent<HTMLInputElement>) => void
  handleSoilChange: (event: React.ChangeEvent<HTMLInputElement>) => void
  handleClusterChange: (event: React.ChangeEvent<HTMLInputElement>) => void
  handlePlantingYearChange: (event: React.ChangeEvent<HTMLInputElement>) => void
  handlePlantingYearRangeChange: (range: number[]) => void
  resetFilters: () => void
  applyOldStateToTags: (oldValues: Filters) => void
}

/* eslint-disable-next-line react-refresh/only-export-components */
export const FilterContext = createContext<FilterContextType | undefined>(undefined)

interface FilterProviderProps {
  children: ReactNode
}

const FilterProvider: React.FC<FilterProviderProps> = ({ children }) => {
  const [statusTags, setStatusTags] = useState<string[]>([])
  const [regionTags, setRegionTags] = useState<string[]>([])
  const [soilTags, setSoilTags] = useState<string[]>([])
  const [plantingYears, setPlantingYears] = useState<number[]>([])
  const [hasCluster, setHasCluster] = useState<boolean | undefined>(undefined)

  const handleStatusChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const { checked, value } = event.target
    setStatusTags((prev) => (checked ? [...prev, value] : prev.filter((tag) => tag !== value)))
  }

  const handleRegionChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const { checked, value } = event.target
    setRegionTags((prev) => (checked ? [...prev, value] : prev.filter((tag) => tag !== value)))
  }

  const handleSoilChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const { checked, value } = event.target
    setSoilTags((prev) => (checked ? [...prev, value] : prev.filter((tag) => tag !== value)))
  }

  const handleClusterChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const { checked, value } = event.target
    if (checked) {
      setHasCluster(value === 'true')
    } else {
      setHasCluster(undefined)
    }
  }

  const handlePlantingYearChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const { checked, value } = event.target
    setPlantingYears((prev) =>
      checked ? [...prev, Number(value)] : prev.filter((year) => year !== Number(value)),
    )
  }

  const handlePlantingYearRangeChange = (range: number[]) => {
    if (range.length !== 2) return
    const [min, max] = range
    const years = Array.from({ length: max - min + 1 }, (_, i) => min + i)
    setPlantingYears(years)
  }

  const applyOldStateToTags = (oldValues: Filters) => {
    setStatusTags(oldValues.statusTags)
    setRegionTags(oldValues.regionTags)
    setSoilTags(oldValues.soilTags)
    setHasCluster(oldValues.hasCluster)
    setPlantingYears(oldValues.plantingYears)
  }

  const resetFilters = () => {
    setStatusTags([])
    setRegionTags([])
    setSoilTags([])
    setHasCluster(undefined)
    setPlantingYears([])
  }

  const context = useMemo(
    () => ({
      filters: { statusTags, regionTags, soilTags, hasCluster, plantingYears },
      handleStatusChange,
      handleRegionChange,
      handleSoilChange,
      handleClusterChange,
      handlePlantingYearChange,
      handlePlantingYearRangeChange,
      resetFilters,
      applyOldStateToTags,
    }),
    [hasCluster, plantingYears, regionTags, soilTags, statusTags],
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
