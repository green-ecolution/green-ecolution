import Option from '../Option'
import { useSuspenseQuery } from '@tanstack/react-query'
import { regionsQuery } from '@/api/queries'
import { useFilter } from '@/context/FilterContext'

const RegionFieldset = () => {
  const { filters, handleRegionChange } = useFilter()
  const { data: regionRes } = useSuspenseQuery(regionsQuery())

  return (
    <fieldset className="mt-6">
      <legend className="font-lato font-semibold text-dark-600 mb-2">
        Stadtteil in Flensburg:
      </legend>
      {regionRes?.data.map((region) => (
        <Option
          key={region.id}
          label={region.name}
          name={String(region.id)}
          checked={filters.regionTags.includes(region.name)}
          onChange={handleRegionChange}
        />
      ))}
    </fieldset>
  )
}

export default RegionFieldset
