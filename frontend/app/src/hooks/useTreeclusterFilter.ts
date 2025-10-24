import { useState } from 'react'

interface Filters {
  statusTags: string[]
  regionTags: string[]
}

const useTreeclusterFilter = (initStatusTags: string[], initRegionTags: string[]) => {
  const [filters, setFilters] = useState<Filters>({
    statusTags: initStatusTags,
    regionTags: initRegionTags,
  })

  const [appliedFilters, setAppliedFilters] = useState<Filters>({
    statusTags: initStatusTags,
    regionTags: initRegionTags,
  })

  const handleFilterChange =
    (type: 'status' | 'region') => (event: React.ChangeEvent<HTMLInputElement>) => {
      const { checked, value } = event.target

      setFilters((prevFilters) => ({
        ...prevFilters,
        [type === 'status' ? 'statusTags' : 'regionTags']: checked
          ? [...prevFilters[type === 'status' ? 'statusTags' : 'regionTags'], value]
          : prevFilters[type === 'status' ? 'statusTags' : 'regionTags'].filter(
              (tag) => tag !== value,
            ),
      }))
    }

  const resetFilters = (applyFilter: (statusTags: string[], regionTags: string[]) => void) => {
    setAppliedFilters({ statusTags: [], regionTags: [] })
    setFilters({ statusTags: [], regionTags: [] })
    applyFilter([], [])
  }

  const applyFilters = (applyFilter: (statusTags: string[], regionTags: string[]) => void) => {
    setAppliedFilters(filters)
    applyFilter(filters.statusTags, filters.regionTags)
  }

  return {
    filters,
    setFilters,
    appliedFilters,
    handleFilterChange,
    resetFilters,
    applyFilters,
  }
}

export default useTreeclusterFilter
