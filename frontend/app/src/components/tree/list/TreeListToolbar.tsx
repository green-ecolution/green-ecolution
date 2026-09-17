import type { ReactNode } from 'react'
import { FolderClosed } from 'lucide-react'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { WateringStatus } from '@green-ecolution/backend-client'
import ListToolbar from '@/components/general/list/ListToolbar'
import ListSearchInput from '@/components/general/list/ListSearchInput'
import ListFilterDropdown from '@/components/general/list/ListFilterDropdown'
import ListSortMenu from '@/components/general/list/ListSortMenu'
import ActiveFilterChips, {
  type FilterChipDescriptor,
} from '@/components/general/list/ActiveFilterChips'
import SensorIcon from '@/components/icons/Sensor'
import { clusterQueries, treeQueries } from '@/api/queries'
import { useWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import { useTreeListSearch, type TreeSortField } from './useTreeListSearch'

interface TreeListToolbarProps {
  filteredRecords: number
  action?: ReactNode
}

const TreeListToolbar = ({ filteredRecords, action }: TreeListToolbarProps) => {
  const { t } = useTranslation('tree')
  const { search, setQuery, setSort, setFilter, setHasSensor, resetFilters } = useTreeListSearch()
  const getWateringStatusDetails = useWateringStatusDetails()

  const { data: clusterPage } = useQuery(clusterQueries.list({ perPage: 100 }))
  const { data: plantingYears } = useQuery(treeQueries.plantingYears())
  // One row is enough: only pagination.totalRecords is read, and react-query
  // serves it from cache for the rest of the session.
  const { data: unfiltered } = useQuery({
    ...treeQueries.list({ page: 1, perPage: 1 }),
    staleTime: 60_000,
  })
  const totalRecords = unfiltered?.pagination?.totalRecords ?? filteredRecords

  const statusOptions = Object.values(WateringStatus).map((status) => ({
    value: status,
    label: getWateringStatusDetails(status).label,
  }))
  const clusterOptions = (clusterPage?.data ?? []).map((cluster) => ({
    value: String(cluster.id),
    label: cluster.name,
  }))
  const yearOptions = (plantingYears ?? []).map((year) => ({
    value: String(year),
    label: String(year),
  }))
  const sensorOptions = [
    { value: 'true', label: t('list.sensorOptionWith') },
    { value: 'false', label: t('list.sensorOptionWithout') },
  ]
  const sortOptions: { value: TreeSortField; label: string }[] = [
    { value: 'number', label: t('list.sortNumber') },
    { value: 'species', label: t('list.sortSpecies') },
    { value: 'status', label: t('list.sortStatus') },
    { value: 'planting_year', label: t('list.sortPlantingYear') },
    { value: 'last_watered', label: t('list.sortLastWatered') },
    { value: 'cluster', label: t('list.sortCluster') },
  ]

  const statuses = search.wateringStatuses ?? []
  const clusterIds = search.clusterIds ?? []
  const years = search.plantingYears ?? []

  const chips: FilterChipDescriptor[] = [
    ...statuses.map((status) => ({
      id: `status-${status}`,
      label: getWateringStatusDetails(status).label,
      onRemove: () =>
        setFilter(
          'wateringStatuses',
          statuses.filter((value) => value !== status),
        ),
    })),
    ...clusterIds.map((id) => ({
      id: `cluster-${id}`,
      label: clusterOptions.find((option) => option.value === id)?.label ?? id,
      onRemove: () =>
        setFilter(
          'clusterIds',
          clusterIds.filter((value) => value !== id),
        ),
    })),
    ...years.map((year) => ({
      id: `year-${year}`,
      label: `${t('list.filterPlantingYear')} ${year}`,
      onRemove: () =>
        setFilter(
          'plantingYears',
          years.filter((value) => value !== year),
        ),
    })),
    ...(search.hasSensor === undefined
      ? []
      : [
          {
            id: 'sensor',
            label: search.hasSensor ? t('list.sensorOptionWith') : t('list.sensorOptionWithout'),
            onRemove: () => setHasSensor(undefined),
          },
        ]),
  ]

  const isFiltered = chips.length > 0 || (search.q ?? '').length > 0
  const resultLabel = isFiltered
    ? t('list.resultCountFiltered', { filtered: filteredRecords, total: totalRecords })
    : t('list.resultCount', { count: totalRecords })

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
              onChange={(values) => setFilter('wateringStatuses', values)}
              emptyText={t('list.filterNoOptions')}
            />
            <ListFilterDropdown
              label={t('list.filterCluster')}
              icon={FolderClosed}
              options={clusterOptions}
              value={clusterIds}
              onChange={(values) => setFilter('clusterIds', values)}
              emptyText={t('list.filterNoOptions')}
            />
            <ListFilterDropdown
              label={t('list.filterSensor')}
              icon={SensorIcon}
              options={sensorOptions}
              value={search.hasSensor === undefined ? [] : [String(search.hasSensor)]}
              onChange={(values) => {
                const picked = values[values.length - 1]
                setHasSensor(picked === undefined ? undefined : picked === 'true')
              }}
              emptyText={t('list.filterNoOptions')}
            />
            <ListFilterDropdown
              label={t('list.filterPlantingYear')}
              options={yearOptions}
              value={years.map(String)}
              onChange={(values) => setFilter('plantingYears', values.map(Number))}
              emptyText={t('list.filterNoOptions')}
            />
          </>
        }
        sort={
          <ListSortMenu
            options={sortOptions}
            field={search.sort ?? 'number'}
            direction={search.order ?? 'asc'}
            onChange={(field, direction) => setSort(field as TreeSortField, direction)}
          />
        }
        action={action}
      />
      <ActiveFilterChips chips={chips} onReset={resetFilters} resultLabel={resultLabel} />
    </>
  )
}

export default TreeListToolbar
