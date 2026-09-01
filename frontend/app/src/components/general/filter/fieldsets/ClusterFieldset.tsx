import { useTranslation } from 'react-i18next'
import useStore from '@/store/store'
import Option from '../Option'

const ClusterFieldset = () => {
  const { t } = useTranslation('treecluster')
  const hasCluster = useStore((s) => s.filterDraft.hasCluster)
  const setHasCluster = useStore((s) => s.setFilterHasCluster)
  const treeClusterOptions = [
    {
      label: t('clusterFieldset.hasCluster'),
      value: true,
    },
    {
      label: t('clusterFieldset.noCluster'),
      value: false,
    },
  ]
  return (
    <fieldset className="mt-4">
      <legend className="font-lato font-semibold text-dark-600 mb-2">
        {t('clusterFieldset.legend')}
      </legend>
      {treeClusterOptions.map((type) => (
        <Option
          key={type.label}
          label={type.label}
          name={type.value.toString()}
          value={String(type.value)}
          checked={hasCluster === type.value}
          onChange={(event) =>
            setHasCluster(event.target.checked ? event.target.value === 'true' : undefined)
          }
        />
      ))}
    </fieldset>
  )
}

export default ClusterFieldset
