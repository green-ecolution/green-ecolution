import { useSuspenseQuery } from '@tanstack/react-query'
import { regionsQuery } from '@/api/queries'
import { useFilter } from '@/context/FilterContext'
import { MultiSelectCombobox } from '@green-ecolution/ui'
import SelectedTagList from '../SelectedTagList'

const RegionFieldset = () => {
  const { filters, setRegionTags } = useFilter()
  const { data: regionRes } = useSuspenseQuery(regionsQuery())

  const options = regionRes.data.map((region) => ({ value: region.id, label: region.name }))

  return (
    <fieldset className="mt-6">
      <legend className="font-lato font-semibold text-dark-600 mb-2">
        Stadtteil in Flensburg:
      </legend>
      <MultiSelectCombobox
        options={options}
        value={filters.regionTags}
        onChange={setRegionTags}
        placeholder="Alle Bezirke"
        searchPlaceholder="Bezirk suchen"
      />
      <SelectedTagList
        options={options}
        value={filters.regionTags}
        onRemove={(v) => setRegionTags(filters.regionTags.filter((tag) => tag !== v))}
      />
    </fieldset>
  )
}

export default RegionFieldset
