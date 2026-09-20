import type { ReactNode } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import {
  ListClustersSortEnum,
  SoilCondition,
  WateringStatus,
} from '@green-ecolution/backend-client'
import ListToolbar from '@/components/general/list/ListToolbar'
import ListSearchInput from '@/components/general/list/ListSearchInput'
import ListFilterDropdown from '@/components/general/list/ListFilterDropdown'
import ListSortMenu from '@/components/general/list/ListSortMenu'
import ActiveFilterChips, {
  type FilterChipDescriptor,
} from '@/components/general/list/ActiveFilterChips'
import { regionsQuery } from '@/api/queries'
import { useWateringStatusDetails } from '@/hooks/details/useDetailsForWateringStatus'
import {
  useSoilConditionLabel,
  useSoilConditionOptions,
} from '@/hooks/details/useDetailsForSoilCondition'
import { useClusterListSearch } from './useClusterListSearch'

interface ClusterListToolbarProps {
  filteredRecords: number
  /** Clusters in scope before the toolbar's own filters; undefined until the list loads. */
  totalRecords?: number
  action?: ReactNode
}

const ClusterListToolbar = ({ filteredRecords, totalRecords, action }: ClusterListToolbarProps) => {
  const { t } = useTranslation('treecluster')
  const { search, setQuery, setSort, setFilter, resetFilters } = useClusterListSearch()
  const getWateringStatusDetails = useWateringStatusDetails()
  const getSoilConditionLabel = useSoilConditionLabel()
  const soilConditionOptions = useSoilConditionOptions()

  const { data: regions } = useQuery(regionsQuery())

  const statusOptions = Object.values(WateringStatus).map((status) => ({
    value: status,
    label: getWateringStatusDetails(status).label,
  }))
  const regionOptions = (regions?.data ?? []).map((region) => ({
    value: String(region.id),
    label: region.name,
  }))
  const soilOptions = soilConditionOptions.map((option) => ({
    value: option.value,
    label: option.label,
  }))
  const sortOptions: { value: ListClustersSortEnum; label: string }[] = [
    { value: ListClustersSortEnum.Name, label: t('list.sortName') },
    { value: ListClustersSortEnum.Moisture, label: t('list.sortMoisture') },
    { value: ListClustersSortEnum.Trees, label: t('list.sortTrees') },
    { value: ListClustersSortEnum.LastWatered, label: t('list.sortLastWatered') },
  ]

  const statuses = search.wateringStatuses ?? []
  const regionIds = search.regions ?? []
  const soils = search.soil ?? []

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
    ...regionIds.map((id) => ({
      id: `region-${id}`,
      label: regionOptions.find((option) => option.value === id)?.label ?? t('list.regionUnknown'),
      onRemove: () =>
        setFilter(
          'regions',
          regionIds.filter((value) => value !== id),
        ),
    })),
    ...soils.map((soil) => ({
      id: `soil-${soil}`,
      label: getSoilConditionLabel(soil as SoilCondition),
      onRemove: () =>
        setFilter(
          'soil',
          soils.filter((value) => value !== soil),
        ),
    })),
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
              onChange={(values) => setFilter('wateringStatuses', values)}
              emptyText={t('list.filterNoOptions')}
            />
            <ListFilterDropdown
              label={t('list.filterRegion')}
              options={regionOptions}
              value={regionIds}
              onChange={(values) => setFilter('regions', values)}
              emptyText={t('list.filterNoOptions')}
            />
            <ListFilterDropdown
              label={t('list.filterSoil')}
              options={soilOptions}
              value={soils}
              onChange={(values) => setFilter('soil', values)}
              emptyText={t('list.filterNoOptions')}
            />
          </>
        }
        sort={
          <ListSortMenu
            options={sortOptions}
            field={search.sort ?? ListClustersSortEnum.Name}
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

export default ClusterListToolbar
