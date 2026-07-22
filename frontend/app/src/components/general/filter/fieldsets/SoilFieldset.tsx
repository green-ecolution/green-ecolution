import useStore from '@/store/store'
import { SoilConditionOptions } from '@/hooks/details/useDetailsForSoilCondition'
import { MultiSelectCombobox } from '@green-ecolution/ui'
import SelectedTagList from '../SelectedTagList'

const SOIL_OPTIONS = SoilConditionOptions.map((option) => ({
  value: option.value,
  label: option.label,
  group: option.group,
}))

const SoilFieldset = () => {
  const soilTags = useStore((s) => s.filterDraft.soilTags)
  const setSoilTags = useStore((s) => s.setFilterSoilTags)

  return (
    <fieldset className="mt-6">
      <legend className="font-lato font-semibold text-dark-600 mb-2">Bodenart:</legend>
      <MultiSelectCombobox
        options={SOIL_OPTIONS}
        value={soilTags}
        onChange={setSoilTags}
        placeholder="Alle Bodenarten"
        searchPlaceholder="Bodenart suchen"
      />
      <SelectedTagList
        options={SOIL_OPTIONS}
        value={soilTags}
        onRemove={(v) => setSoilTags(soilTags.filter((tag) => tag !== v))}
      />
    </fieldset>
  )
}

export default SoilFieldset
