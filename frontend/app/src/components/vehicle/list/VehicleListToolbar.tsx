import type { ReactNode } from 'react'
import { Archive } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  DrivingLicense,
  ListVehiclesArchiveEnum,
  ListVehiclesSortEnum,
  VehicleStatus,
  VehicleType,
} from '@green-ecolution/backend-client'
import ListToolbar from '@/components/general/list/ListToolbar'
import ListSearchInput from '@/components/general/list/ListSearchInput'
import ListFilterDropdown from '@/components/general/list/ListFilterDropdown'
import ListSortMenu from '@/components/general/list/ListSortMenu'
import ActiveFilterChips, {
  type FilterChipDescriptor,
} from '@/components/general/list/ActiveFilterChips'
import { useVehicleStatusDetails } from '@/hooks/details/useDetailsForVehicleStatus'
import { useVehicleTypeLabel } from '@/hooks/details/useDetailsForVehicleType'
import { useVehicleListSearch } from './useVehicleListSearch'

interface VehicleListToolbarProps {
  filteredRecords: number
  /** Vehicles in scope before the toolbar's own filters; undefined until the list loads. */
  totalRecords?: number
  action?: ReactNode
}

const VehicleListToolbar = ({ filteredRecords, totalRecords, action }: VehicleListToolbarProps) => {
  const { t } = useTranslation('vehicle')
  const { search, setQuery, setSort, setFilter, setArchive, resetFilters } = useVehicleListSearch()
  const getVehicleStatusDetails = useVehicleStatusDetails()
  const getVehicleTypeLabel = useVehicleTypeLabel()

  const statusOptions = Object.values(VehicleStatus).map((status) => ({
    value: status,
    label: getVehicleStatusDetails(status).label,
  }))
  const typeOptions = Object.values(VehicleType).map((type) => ({
    value: type,
    label: getVehicleTypeLabel(type),
  }))
  const drivingLicenseOptions = Object.values(DrivingLicense).map((license) => ({
    value: license,
    label: license,
  }))
  const sortOptions: { value: ListVehiclesSortEnum; label: string }[] = [
    { value: ListVehiclesSortEnum.NumberPlate, label: t('list.sortNumberPlate') },
    { value: ListVehiclesSortEnum.WaterCapacity, label: t('list.sortWaterCapacity') },
    { value: ListVehiclesSortEnum.Model, label: t('list.sortModel') },
    { value: ListVehiclesSortEnum.Type, label: t('list.sortType') },
  ]

  const statuses = search.statuses ?? []
  const types = search.types ?? []
  const drivingLicenses = search.drivingLicenses ?? []

  const chips: FilterChipDescriptor[] = [
    ...statuses.map((status) => ({
      id: `status-${status}`,
      label: getVehicleStatusDetails(status).label,
      onRemove: () =>
        setFilter(
          'statuses',
          statuses.filter((value) => value !== status),
        ),
    })),
    ...types.map((type) => ({
      id: `type-${type}`,
      label: getVehicleTypeLabel(type),
      onRemove: () =>
        setFilter(
          'types',
          types.filter((value) => value !== type),
        ),
    })),
    ...drivingLicenses.map((license) => ({
      id: `driving-license-${license}`,
      label: license,
      onRemove: () =>
        setFilter(
          'drivingLicenses',
          drivingLicenses.filter((value) => value !== license),
        ),
    })),
    ...(search.archive
      ? [
          {
            id: `archive-${search.archive}`,
            label:
              search.archive === ListVehiclesArchiveEnum.Only
                ? t('list.archiveOnly')
                : t('list.archiveInclude'),
            onRemove: () => setArchive(undefined),
          },
        ]
      : []),
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
              label={t('list.filterType')}
              options={typeOptions}
              value={types}
              onChange={(values) => setFilter('types', values)}
              emptyText={t('list.filterNoOptions')}
            />
            <ListFilterDropdown
              label={t('list.filterDrivingLicense')}
              options={drivingLicenseOptions}
              value={drivingLicenses}
              onChange={(values) => setFilter('drivingLicenses', values)}
              emptyText={t('list.filterNoOptions')}
            />
            <ListFilterDropdown
              label={t('list.filterArchive')}
              mode="single"
              icon={Archive}
              options={[
                { value: ListVehiclesArchiveEnum.Include, label: t('list.archiveInclude') },
                { value: ListVehiclesArchiveEnum.Only, label: t('list.archiveOnly') },
              ]}
              value={search.archive ? [search.archive] : []}
              onChange={(values) => setArchive(values[0] as ListVehiclesArchiveEnum | undefined)}
              emptyText={t('list.filterNoOptions')}
            />
          </>
        }
        sort={
          <ListSortMenu
            options={sortOptions}
            field={search.sort ?? ListVehiclesSortEnum.NumberPlate}
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

export default VehicleListToolbar
