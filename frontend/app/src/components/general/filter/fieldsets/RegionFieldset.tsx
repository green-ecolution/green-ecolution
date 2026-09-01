import { useSuspenseQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { regionsQuery } from '@/api/queries'
import useStore from '@/store/store'
import { MultiSelectCombobox } from '@green-ecolution/ui'
import SelectedTagList from '../SelectedTagList'

const RegionFieldset = () => {
  const { t } = useTranslation('treecluster')
  const regionTags = useStore((s) => s.filterDraft.regionTags)
  const setRegionTags = useStore((s) => s.setFilterRegionTags)
  const { data: regionRes } = useSuspenseQuery(regionsQuery())

  const options = regionRes.data.map((region) => ({ value: region.id, label: region.name }))

  return (
    <fieldset className="mt-6">
      <legend className="font-lato font-semibold text-dark-600 mb-2">
        {t('regionFieldset.legend')}
      </legend>
      <MultiSelectCombobox
        options={options}
        value={regionTags}
        onChange={setRegionTags}
        placeholder={t('regionFieldset.placeholder')}
        searchPlaceholder={t('regionFieldset.searchPlaceholder')}
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
