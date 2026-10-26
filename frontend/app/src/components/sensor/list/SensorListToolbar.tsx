import { Suspense, type ReactNode } from 'react'
import { Link2 } from 'lucide-react'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { DataHealth, ListSensorsSortEnum } from '@green-ecolution/backend-client'
import ListToolbar from '@/components/general/list/ListToolbar'
import ListSearchInput from '@/components/general/list/ListSearchInput'
import ListFilterDropdown from '@/components/general/list/ListFilterDropdown'
import ListSortMenu from '@/components/general/list/ListSortMenu'
import ActiveFilterChips, {
  type FilterChipDescriptor,
} from '@/components/general/list/ActiveFilterChips'
import { sensorQueries } from '@/api/queries'
import { listFilterValue, sensorListFiltersOf, soleTargetKind, targetTextKey } from '@/lib/targets'
import { useActiveModules } from '@/modules/useActiveModules'
import { useSensorStatusDetails } from '@/hooks/details/useDetailsForSensorStatus'
import { useDataQualityDetails } from '@/hooks/details/useDetailsForDataHealth'
import { useSensorListSearch } from './useSensorListSearch'
import { ACTIVATED_STATUSES } from './sensorListView'

interface SensorListToolbarProps {
  filteredRecords: number
  /** Sensors in scope before the toolbar's own filters; undefined until the list loads. */
  totalRecords?: number
  action?: ReactNode
}

const SensorListToolbar = ({ filteredRecords, totalRecords, action }: SensorListToolbarProps) => {
  const { t, i18n } = useTranslation('sensor')
  const { search, setQuery, setSort, setFilter, setAttached, resetFilters } = useSensorListSearch()
  const getSensorStatusDetails = useSensorStatusDetails()
  const getDataQualityDetails = useDataQualityDetails()
  const { manifests } = useActiveModules()
  const targetFilters = sensorListFiltersOf(manifests).map(({ module, kind, filter }) => ({
    key: `${module}:${kind}`,
    filter,
    value: listFilterValue(search, filter.param),
  }))

  const { data: models } = useQuery(sensorQueries.models())

  const isPreparedView = search.view === 'prepared'

  const statusOptions = ACTIVATED_STATUSES.map((status) => ({
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
  const attachedOptions = [
    { value: 'true', label: t('list.attachedOptionWith') },
    { value: 'false', label: t('list.attachedOptionWithout') },
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
    ...(search.attached === undefined
      ? []
      : [
          {
            id: 'attached',
            label: search.attached ? t('list.attachedOptionWith') : t('list.attachedOptionWithout'),
            onRemove: () => setAttached(undefined),
          },
        ]),
  ]
  const targetFilterSet = !isPreparedView && targetFilters.some(({ value }) => value.length > 0)

  const isFiltered = chips.length > 0 || targetFilterSet || (search.q ?? '').length > 0
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
            placeholder={i18n.t(
              targetTextKey(
                soleTargetKind(manifests),
                'listSearchPlaceholder',
                'sensor:list.searchPlaceholder',
              ),
            )}
          />
        }
        filters={
          <>
            {!isPreparedView && (
              <ListFilterDropdown
                label={t('list.filterStatus')}
                options={statusOptions}
                value={statuses}
                onChange={(values) => setFilter('statuses', values)}
                emptyText={t('list.filterNoOptions')}
              />
            )}
            <ListFilterDropdown
              label={t('list.filterModel')}
              options={modelOptions}
              value={modelIds}
              onChange={(values) => setFilter('modelIds', values)}
              emptyText={t('list.filterNoOptions')}
            />
            {!isPreparedView && (
              <>
                <ListFilterDropdown
                  label={t('list.filterDataHealth')}
                  options={dataHealthOptions}
                  value={dataHealth}
                  onChange={(values) => setFilter('dataHealth', values)}
                  emptyText={t('list.filterNoOptions')}
                />
                <ListFilterDropdown
                  label={t('list.filterAttached')}
                  icon={Link2}
                  mode="single"
                  options={attachedOptions}
                  value={search.attached === undefined ? [] : [String(search.attached)]}
                  onChange={(values) =>
                    setAttached(values[0] === undefined ? undefined : values[0] === 'true')
                  }
                  emptyText={t('list.filterNoOptions')}
                />
                {targetFilters.map(({ key, filter: { param, Control }, value }) => (
                  <Suspense key={key} fallback={null}>
                    <Control value={value} onChange={(values) => setFilter(param, values)} />
                  </Suspense>
                ))}
              </>
            )}
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
      <ActiveFilterChips
        chips={chips}
        active={targetFilterSet}
        onReset={resetFilters}
        resultLabel={resultLabel}
      >
        {!isPreparedView &&
          targetFilters.map(({ key, filter: { param, Chips }, value }) => (
            <Suspense key={key} fallback={null}>
              <Chips value={value} onChange={(values) => setFilter(param, values)} />
            </Suspense>
          ))}
      </ActiveFilterChips>
    </>
  )
}

export default SensorListToolbar
