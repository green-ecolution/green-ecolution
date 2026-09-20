import type { ReactNode } from 'react'
import { FolderClosed, TreeDeciduous } from 'lucide-react'
import { useQueries, useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { DataHealth, ListSensorsSortEnum, SensorStatus } from '@green-ecolution/backend-client'
import ListToolbar from '@/components/general/list/ListToolbar'
import ListSearchInput from '@/components/general/list/ListSearchInput'
import ListFilterDropdown from '@/components/general/list/ListFilterDropdown'
import ListSortMenu from '@/components/general/list/ListSortMenu'
import ActiveFilterChips, {
  type FilterChipDescriptor,
} from '@/components/general/list/ActiveFilterChips'
import { clusterQueries, sensorQueries } from '@/api/queries'
import { useSensorStatusDetails } from '@/hooks/details/useDetailsForSensorStatus'
import { useDataQualityDetails } from '@/hooks/details/useDetailsForDataHealth'
import { useSensorListSearch } from './useSensorListSearch'

interface SensorListToolbarProps {
  filteredRecords: number
  /** Sensors in scope before the toolbar's own filters; undefined until the list loads. */
  totalRecords?: number
  action?: ReactNode
}

const SensorListToolbar = ({ filteredRecords, totalRecords, action }: SensorListToolbarProps) => {
  const { t } = useTranslation('sensor')
  const { search, setQuery, setSort, setFilter, setHasTree, resetFilters } = useSensorListSearch()
  const getSensorStatusDetails = useSensorStatusDetails()
  const getDataQualityDetails = useDataQualityDetails()

  const { data: models } = useQuery(sensorQueries.models())
  const { data: clusterPage } = useQuery(clusterQueries.list({ perPage: 100 }))

  const statusOptions = Object.values(SensorStatus).map((status) => ({
    value: status,
    label: getSensorStatusDetails(status).label,
  }))
  const modelOptions = (models ?? []).map((model) => ({
    value: model.id,
    label: model.name,
  }))
  const dataHealthOptions = Object.values(DataHealth).map((health) => ({
    value: health,
    label: getDataQualityDetails({ dataHealth: health, implausibleRecent: 0 }).label,
  }))
  const clusterOptions = (clusterPage?.data ?? []).map((cluster) => ({
    value: String(cluster.id),
    label: cluster.name,
  }))
  // A cluster selected via a shared link can sit outside the first 100
  // clusters (backend's Pagination::MAX_PER_PAGE), so its name never reaches
  // clusterOptions; resolve it individually rather than showing the raw id.
  const missingClusterIds = (search.clusterIds ?? []).filter(
    (id) => !clusterOptions.some((option) => option.value === id),
  )
  const missingClusterQueries = useQueries({
    queries: missingClusterIds.map((id) => clusterQueries.detail(id)),
  })
  const missingClusterNames = new Map(
    missingClusterIds
      .map((id, index) => [id, missingClusterQueries[index]?.data?.name] as const)
      .filter((entry): entry is [string, string] => entry[1] !== undefined),
  )
  const treeOptions = [
    { value: 'true', label: t('list.treeOptionWith') },
    { value: 'false', label: t('list.treeOptionWithout') },
  ]
  // sort=status is deliberately not offered: the backend ranks activated-but-
  // never-reported sensors first, which does not match the displayed status
  // (derived from reading recency), so the visible order would mislead.
  const sortOptions: { value: ListSensorsSortEnum; label: string }[] = [
    { value: ListSensorsSortEnum.Id, label: t('list.sortId') },
    { value: ListSensorsSortEnum.LastReading, label: t('list.sortLastReading') },
    { value: ListSensorsSortEnum.CreatedAt, label: t('list.sortCreatedAt') },
  ]

  const statuses = search.statuses ?? []
  const modelIds = search.modelIds ?? []
  const dataHealth = search.dataHealth ?? []
  const clusterIds = search.clusterIds ?? []

  const chips: FilterChipDescriptor[] = [
    ...statuses.map((status) => ({
      id: `status-${status}`,
      label: getSensorStatusDetails(status).label,
      onRemove: () =>
        setFilter(
          'statuses',
          statuses.filter((value) => value !== status),
        ),
    })),
    ...modelIds.map((id) => ({
      id: `model-${id}`,
      label: modelOptions.find((option) => option.value === id)?.label ?? t('card.modelUnknown'),
      onRemove: () =>
        setFilter(
          'modelIds',
          modelIds.filter((value) => value !== id),
        ),
    })),
    ...dataHealth.map((health) => ({
      id: `data-health-${health}`,
      label: getDataQualityDetails({ dataHealth: health, implausibleRecent: 0 }).label,
      onRemove: () =>
        setFilter(
          'dataHealth',
          dataHealth.filter((value) => value !== health),
        ),
    })),
    ...clusterIds.map((id) => ({
      id: `cluster-${id}`,
      label:
        clusterOptions.find((option) => option.value === id)?.label ??
        missingClusterNames.get(id) ??
        t('list.clusterUnknown'),
      onRemove: () =>
        setFilter(
          'clusterIds',
          clusterIds.filter((value) => value !== id),
        ),
    })),
    ...(search.hasTree === undefined
      ? []
      : [
          {
            id: 'tree',
            label: search.hasTree ? t('list.treeOptionWith') : t('list.treeOptionWithout'),
            onRemove: () => setHasTree(undefined),
          },
        ]),
  ]

  const isFiltered = chips.length > 0 || (search.q ?? '').length > 0
  const resultLabel =
    isFiltered && totalRecords !== undefined
      ? t('list.resultCountFiltered', { filtered: filteredRecords, total: totalRecords })
      : t('list.resultCount', { count: totalRecords ?? filteredRecords })

  return (
    <>
      <ListToolbar
        search={
          <ListSearchInput
            value={search.q ?? ''}
            onChange={setQuery}
            label={t('list.searchLabel')}
            placeholder={t('list.searchPlaceholder')}
          />
        }
        filters={
          <>
            <ListFilterDropdown
              label={t('list.filterStatus')}
              options={statusOptions}
              value={statuses}
              onChange={(values) => setFilter('statuses', values)}
              emptyText={t('list.filterNoOptions')}
            />
            <ListFilterDropdown
              label={t('list.filterModel')}
              options={modelOptions}
              value={modelIds}
              onChange={(values) => setFilter('modelIds', values)}
              emptyText={t('list.filterNoOptions')}
            />
            <ListFilterDropdown
              label={t('list.filterDataHealth')}
              options={dataHealthOptions}
              value={dataHealth}
              onChange={(values) => setFilter('dataHealth', values)}
              emptyText={t('list.filterNoOptions')}
            />
            <ListFilterDropdown
              label={t('list.filterTree')}
              icon={TreeDeciduous}
              mode="single"
              options={treeOptions}
              value={search.hasTree === undefined ? [] : [String(search.hasTree)]}
              onChange={(values) =>
                setHasTree(values[0] === undefined ? undefined : values[0] === 'true')
              }
              emptyText={t('list.filterNoOptions')}
            />
            {/* No 'any' / 'none' options: whether a sensor is linked at all is
                what the tree filter next to it already answers. */}
            <ListFilterDropdown
              label={t('list.filterCluster')}
              icon={FolderClosed}
              options={clusterOptions}
              value={clusterIds}
              onChange={(values) => setFilter('clusterIds', values)}
              emptyText={t('list.filterNoOptions')}
            />
          </>
        }
        sort={
          <ListSortMenu
            options={sortOptions}
            field={search.sort ?? ListSensorsSortEnum.Id}
            direction={search.order ?? 'asc'}
            onChange={setSort}
          />
        }
        action={action}
      />
      <ActiveFilterChips chips={chips} onReset={resetFilters} resultLabel={resultLabel} />
    </>
  )
}

export default SensorListToolbar
