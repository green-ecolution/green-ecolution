import { FolderClosed } from 'lucide-react'
import { useQueries, useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { FilterChip } from '@green-ecolution/ui'
import ListFilterDropdown from '@/components/general/list/ListFilterDropdown'
import type { SensorListFilterProps } from '@/modules/types'
import { clusterQueries } from '../../../api/queries'

const useClusterOptions = (selectedIds: string[]) => {
  const { data: clusterPage } = useQuery(clusterQueries.list({ perPage: 100 }))
  const options = (clusterPage?.data ?? []).map((cluster) => ({
    value: String(cluster.id),
    label: cluster.name,
  }))
  // A cluster selected via a shared link can sit outside the first 100
  // clusters (backend's Pagination::MAX_PER_PAGE), so its name never reaches
  // the options; resolve it individually rather than showing the raw id.
  const missingIds = selectedIds.filter((id) => !options.some((option) => option.value === id))
  const missing = useQueries({ queries: missingIds.map((id) => clusterQueries.detail(id)) })
  const missingNames = new Map(
    missingIds
      .map((id, index) => [id, missing[index]?.data?.name] as const)
      .filter((entry): entry is [string, string] => entry[1] !== undefined),
  )
  const labelOf = (id: string) =>
    options.find((option) => option.value === id)?.label ?? missingNames.get(id)
  return { options, labelOf }
}

export const ClusterSensorFilterControl = ({ value, onChange }: SensorListFilterProps) => {
  const { t } = useTranslation(['irrigation', 'sensor'])
  const { options } = useClusterOptions(value)
  // No 'any' / 'none' options: whether a sensor is linked at all is what the
  // core's link filter next to it already answers.
  return (
    <ListFilterDropdown
      label={t('sensor.list.filterCluster')}
      icon={FolderClosed}
      options={options}
      value={value}
      onChange={onChange}
      emptyText={t('sensor:list.filterNoOptions')}
    />
  )
}

export const ClusterSensorFilterChips = ({ value, onChange }: SensorListFilterProps) => {
  const { t } = useTranslation('irrigation')
  const { labelOf } = useClusterOptions(value)
  return value.map((id) => (
    <FilterChip
      key={id}
      label={labelOf(id) ?? t('sensor.list.clusterUnknown')}
      onRemove={() => onChange(value.filter((other) => other !== id))}
    />
  ))
}
