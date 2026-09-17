import type { ReactNode } from 'react'
import { FolderClosed } from 'lucide-react'
import { useQueries, useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { ListTreesSortEnum, WateringStatus } from '@green-ecolution/backend-client'
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
import { useTreeListSearch } from './useTreeListSearch'

interface TreeListToolbarProps {
  filteredRecords: number
  action?: ReactNode
}

const TreeListToolbar = ({ filteredRecords, action }: TreeListToolbarProps) => {
  const { t } = useTranslation('tree')
  const {
    search,
    setQuery,
    setSort,
    setFilter,
    setHasSensor,
    setHasCluster,
    setClusterAssignment,
    resetFilters,
  } = useTreeListSearch()
  const getWateringStatusDetails = useWateringStatusDetails()

  const { data: clusterPage } = useQuery(clusterQueries.list({ perPage: 100 }))
  const { data: plantingYears } = useQuery(treeQueries.plantingYears())
  // One row is enough: only pagination.totalRecords is read, and react-query
  // serves it from cache for the rest of the session.
  const { data: unfiltered } = useQuery({
    ...treeQueries.list({ page: 1, perPage: 1 }),
    staleTime: 60_000,
  })
  const totalRecords = unfiltered?.pagination?.totalRecords

  const statusOptions = Object.values(WateringStatus).map((status) => ({
    value: status,
    label: getWateringStatusDetails(status).label,
  }))
  const clusterOptions = (clusterPage?.data ?? []).map((cluster) => ({
    value: String(cluster.id),
    label: cluster.name,
  }))
  // A cluster selected via a shared link can sit outside the first 100
  // clusters (backend's Pagination::MAX_PER_PAGE), so its name never
  // reaches clusterOptions; resolve it individually rather than showing the
  // raw id.
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
  // 'any' / 'none' answer "is this tree in a cluster at all", not a narrowing
  // of the cluster list, so they sit ahead of the real clusters.
  const clusterDropdownOptions = [
    { value: 'any', label: t('list.clusterOptionAny') },
    { value: 'none', label: t('list.clusterOptionNone') },
    ...clusterOptions,
  ]
  const yearOptions = (plantingYears ?? []).map((year) => ({
    value: String(year),
    label: String(year),
  }))
  const sensorOptions = [
    { value: 'true', label: t('list.sensorOptionWith') },
    { value: 'false', label: t('list.sensorOptionWithout') },
  ]
  const sortOptions: { value: ListTreesSortEnum; label: string }[] = [
    { value: ListTreesSortEnum.Number, label: t('list.sortNumber') },
    { value: ListTreesSortEnum.Species, label: t('list.sortSpecies') },
    { value: ListTreesSortEnum.Status, label: t('list.sortStatus') },
    { value: ListTreesSortEnum.PlantingYear, label: t('list.sortPlantingYear') },
    { value: ListTreesSortEnum.LastWatered, label: t('list.sortLastWatered') },
    { value: ListTreesSortEnum.Cluster, label: t('list.sortCluster') },
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
    ...(search.hasCluster === undefined
      ? []
      : [
          {
            id: 'cluster-assignment',
            label: search.hasCluster ? t('list.clusterOptionAny') : t('list.clusterOptionNone'),
            onRemove: () => setHasCluster(undefined),
          },
        ]),
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
  // Before the unfiltered total loads, showing the plain count avoids a
  // flash of e.g. "17 of 17" comparing filteredRecords against itself.
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
              onChange={(values) => setFilter('wateringStatuses', values)}
              emptyText={t('list.filterNoOptions')}
            />
            <ListFilterDropdown
              label={t('list.filterCluster')}
              icon={FolderClosed}
              options={clusterDropdownOptions}
              value={
                search.hasCluster === true
                  ? ['any']
                  : search.hasCluster === false
                    ? ['none']
                    : clusterIds
              }
              onChange={(values) => {
                const last = values[values.length - 1]
                if (last === 'any') {
                  setClusterAssignment({ hasCluster: true })
                } else if (last === 'none') {
                  setClusterAssignment({ hasCluster: false })
                } else {
                  setClusterAssignment({
                    clusterIds: values.filter((value) => value !== 'any' && value !== 'none'),
                  })
                }
              }}
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
            field={search.sort ?? ListTreesSortEnum.Number}
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

export default TreeListToolbar
