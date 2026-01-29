import { useFilter } from '@/context/FilterContext'
import { Slider } from '@green-ecolution/ui'
import { useMemo } from 'react'

const PlantingYearFieldset = () => {
  const { filters, handlePlantingYearRangeChange } = useFilter()
  const currentYear = new Date().getFullYear()
  const minYear = currentYear - 10

  const range = useMemo(() => {
    if (filters.plantingYears.length === 0) {
      return [minYear, currentYear]
    }
    const sortedYears = [...filters.plantingYears].sort((a, b) => a - b)
    return [sortedYears[0], sortedYears[sortedYears.length - 1]]
  }, [filters.plantingYears, minYear, currentYear])

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
          max={currentYear}
          step={1}
          showLabels
        />
      </div>
    </fieldset>
  )
}

export default PlantingYearFieldset
