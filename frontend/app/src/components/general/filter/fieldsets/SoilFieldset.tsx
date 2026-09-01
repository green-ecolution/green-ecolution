import { useTranslation } from 'react-i18next'
import useStore from '@/store/store'
import { useSoilConditionOptions } from '@/hooks/details/useDetailsForSoilCondition'
import { MultiSelectCombobox } from '@green-ecolution/ui'
import SelectedTagList from '../SelectedTagList'

const SoilFieldset = () => {
  const { t } = useTranslation('treecluster')
  const soilTags = useStore((s) => s.filterDraft.soilTags)
  const setSoilTags = useStore((s) => s.setFilterSoilTags)
  const SOIL_OPTIONS = useSoilConditionOptions()

  return (
    <fieldset className="mt-6">
      <legend className="font-lato font-semibold text-dark-600 mb-2">
        {t('soilFieldset.legend')}
      </legend>
      <MultiSelectCombobox
        options={SOIL_OPTIONS}
        value={soilTags}
        onChange={setSoilTags}
        placeholder={t('soilFieldset.placeholder')}
        searchPlaceholder={t('soilFieldset.searchPlaceholder')}
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
