import { useFilter } from '@/context/FilterContext'
import Option from '../Option'

const PlantingYearFieldset = () => {
  const { filters, handlePlantingYearChange } = useFilter()
  const currentYear = new Date().getFullYear()
  const years = Array.from({ length: 5 }, (_, i) => currentYear - i)

  return (
    <fieldset className="mt-4">
      <legend className="font-lato font-semibold text-dark-600 mb-2">Pflanzjahr:</legend>
      {years.map((year) => (
        <Option
          key={year}
          label={String(year)}
          name={String(year)}
          checked={filters.plantingYears.includes(year)}
          onChange={handlePlantingYearChange}
        />
      ))}
    </fieldset>
  )
}

export default PlantingYearFieldset
