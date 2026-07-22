import useStore from '@/store/store'
import { Slider } from '@green-ecolution/ui'
import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { plantingYearsQuery } from '@/api/queries'

const PlantingYearFieldset = () => {
  const plantingYears = useStore((s) => s.filterDraft.plantingYears)
  const setPlantingYearRange = useStore((s) => s.setFilterPlantingYearRange)
  const { data: availableYears, isLoading } = useQuery(plantingYearsQuery())

  const { minYear, maxYear } = useMemo(() => {
    if (!availableYears || availableYears.length === 0) {
      const currentYear = new Date().getFullYear()
      return { minYear: currentYear - 4, maxYear: currentYear }
    }
    return {
      minYear: Math.min(...availableYears),
      maxYear: Math.max(...availableYears),
    }
  }, [availableYears])

  const range = useMemo(() => {
    if (plantingYears.length === 0) {
      return [minYear, maxYear]
    }
    const sortedYears = [...plantingYears].sort((a, b) => a - b)
    return [sortedYears[0], sortedYears[sortedYears.length - 1]]
  }, [plantingYears, minYear, maxYear])

  if (isLoading) {
    return (
      <fieldset className="mt-4">
        <legend className="font-lato font-semibold text-dark-600 mb-2">Pflanzjahr:</legend>
        <p className="text-sm text-dark-400">Lädt...</p>
      </fieldset>
    )
  }

  if (minYear === maxYear) {
    return null
  }

  return (
    <fieldset className="mt-4">
      <legend className="font-lato font-semibold text-dark-600 mb-2">
        Pflanzjahr: {range[0]} - {range[1]}
      </legend>
      <div className="px-1">
        <Slider
          value={range}
          onValueChange={setPlantingYearRange}
          min={minYear}
          max={maxYear}
          step={1}
          showLabels
        />
      </div>
    </fieldset>
  )
}

export default PlantingYearFieldset
