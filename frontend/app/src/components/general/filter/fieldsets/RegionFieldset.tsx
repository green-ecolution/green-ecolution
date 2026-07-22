import { useSuspenseQuery } from '@tanstack/react-query'
import { regionsQuery } from '@/api/queries'
import useStore from '@/store/store'
import { MultiSelectCombobox } from '@green-ecolution/ui'
import SelectedTagList from '../SelectedTagList'

const RegionFieldset = () => {
  const regionTags = useStore((s) => s.filterDraft.regionTags)
  const setRegionTags = useStore((s) => s.setFilterRegionTags)
  const { data: regionRes } = useSuspenseQuery(regionsQuery())

  const options = regionRes.data.map((region) => ({ value: region.id, label: region.name }))

  return (
    <fieldset className="mt-6">
      <legend className="font-lato font-semibold text-dark-600 mb-2">
        Stadtteil in Flensburg:
      </legend>
      <MultiSelectCombobox
        options={options}
        value={regionTags}
        onChange={setRegionTags}
        placeholder="Alle Bezirke"
        searchPlaceholder="Bezirk suchen"
      />
      <SelectedTagList
        options={options}
        value={regionTags}
        onRemove={(v) => setRegionTags(regionTags.filter((tag) => tag !== v))}
      />
    </fieldset>
  )
}

export default RegionFieldset
