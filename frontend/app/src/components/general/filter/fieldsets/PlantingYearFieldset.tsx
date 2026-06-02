import { useFilter } from '@/context/FilterContext'
import { Slider } from '@green-ecolution/ui'
import { useQuery } from '@tanstack/react-query'
import { plantingYearsQuery } from '@/api/queries'

const PlantingYearFieldset = () => {
  const { filters, handlePlantingYearRangeChange } = useFilter()
  const { data: availableYears, isLoading } = useQuery(plantingYearsQuery())

  const yearBounds =
    !availableYears || availableYears.length === 0
      ? (() => {
          const currentYear = new Date().getFullYear()
          return { minYear: currentYear - 4, maxYear: currentYear }
        })()
      : {
          minYear: Math.min(...availableYears),
          maxYear: Math.max(...availableYears),
        }
  const { minYear, maxYear } = yearBounds

  const sortedYears = filters.plantingYears.toSorted((a, b) => a - b)
  const range =
    filters.plantingYears.length === 0
      ? [minYear, maxYear]
      : [sortedYears[0], sortedYears[sortedYears.length - 1]]

  if (isLoading) {
    return (
      <fieldset className="mt-4">
        <legend className="font-lato font-semibold text-dark-600 mb-2">Pflanzjahr:</legend>
        <p className="text-sm text-dark-400">Lädt…</p>
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
          onValueChange={handlePlantingYearRangeChange}
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
