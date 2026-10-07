import { sensorQueries } from '@/api/queries'
import { Badge, Button, Loading, Tabs, TabsList, TabsTrigger } from '@green-ecolution/ui'
import Pagination from '@/components/general/Pagination'
import EntityList from '@/components/general/EntityList'
import SensorCard from '@/components/general/cards/SensorCard'
import SensorListToolbar from '@/components/sensor/list/SensorListToolbar'
import { useSensorListSearch } from '@/components/sensor/list/useSensorListSearch'
import {
  ACTIVATED_STATUSES,
  legacyPreparedRedirect,
  sensorFilterSchema,
  statusesForView,
  type SensorListSearch,
  type SensorListView,
} from '@/components/sensor/list/sensorListView'
import { useQuery, keepPreviousData } from '@tanstack/react-query'
import { createFileRoute, Link, redirect } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { Zap } from 'lucide-react'
import { SensorStatus } from '@green-ecolution/backend-client'
import { pendingLoading, prefetch } from '@/lib/router'
import { Can } from '@/lib/auth/Can'

export const PER_PAGE = 25

const listParams = (search: SensorListSearch) => ({
  page: search.page,
  perPage: PER_PAGE,
  q: search.q,
  status: statusesForView(search.view ?? 'activated', search.statuses),
  modelId: search.modelIds,
  dataHealth: search.dataHealth,
  hasTree: search.hasTree,
  clusterId: search.clusterIds,
  sort: search.sort,
  order: search.order,
})

const viewTotalParams = (statuses: SensorStatus[]) => ({ page: 1, perPage: 1, status: statuses })

const useViewTotal = (statuses: SensorStatus[]) => {
  const { data } = useQuery(sensorQueries.list(viewTotalParams(statuses)))
  return data?.pagination?.totalRecords
}

function Sensors() {
  const { t } = useTranslation('sensor')
  const search = Route.useSearch()
  const view = search.view ?? 'activated'
  const { resetFilters, setView } = useSensorListSearch()
  const {
    data: sensorsRes,
    isPlaceholderData,
    error,
  } = useQuery({
    ...sensorQueries.list(listParams(search)),
    placeholderData: keepPreviousData,
  })
  if (error) throw error

  const activatedTotal = useViewTotal(ACTIVATED_STATUSES)
  const preparedTotal = useViewTotal([SensorStatus.Prepared])

  const isFiltered =
    (search.q ?? '').length > 0 ||
    (search.statuses?.length ?? 0) > 0 ||
    (search.modelIds?.length ?? 0) > 0 ||
    (search.dataHealth?.length ?? 0) > 0 ||
    (search.clusterIds?.length ?? 0) > 0 ||
    search.hasTree !== undefined

  const viewTabs: { value: SensorListView; label: string; total: number | undefined }[] = [
    { value: 'activated', label: t('list.viewActivated'), total: activatedTotal },
    { value: 'prepared', label: t('list.viewPrepared'), total: preparedTotal },
  ]

  return (
    <div className="container mt-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <article className="flex-1">
          <h1 className="font-lato font-bold text-3xl mb-2 lg:text-4xl xl:text-5xl">
            {t('list.title')}
          </h1>
          <p className="text-sm text-muted-foreground max-w-prose">{t('list.description')}</p>
        </article>
      </div>

      <Tabs
        value={view}
        onValueChange={(value) => {
          if (value === 'activated' || value === 'prepared') setView(value)
        }}
        className="mt-8"
      >
        <TabsList aria-label={t('list.viewsAriaLabel')}>
          {viewTabs.map((tab) => (
            <TabsTrigger key={tab.value} value={tab.value}>
              {tab.label}
              {tab.total !== undefined && <Badge variant="muted">{tab.total}</Badge>}
            </TabsTrigger>
          ))}
        </TabsList>
      </Tabs>

      <section className="mt-6">
        <SensorListToolbar
          filteredRecords={sensorsRes?.pagination?.totalRecords ?? 0}
          totalRecords={view === 'prepared' ? preparedTotal : activatedTotal}
          action={
            <Can permission={['sensor:create']}>
              <Button asChild size="sm" className="w-full sm:w-auto sm:shrink-0">
                <Link to="/sensors/new">
                  <Zap />
                  {t('list.activateButton')}
                </Link>
              </Button>
            </Can>
          }
        />

        {!sensorsRes ? (
          <Loading className="mt-10 justify-center" label={t('list.loadingLabel')} />
        ) : (
          <div
            className="mt-6 transition-opacity duration-200"
            style={{ opacity: isPlaceholderData ? 0.6 : 1 }}
            aria-busy={isPlaceholderData}
          >
            {sensorsRes.data.length === 0 && isFiltered ? (
              <div className="mt-10 text-center">
                <p className="text-dark-600">{t('list.emptyFilteredMessage')}</p>
                <Button variant="outline" className="mt-4" onClick={resetFilters}>
                  {t('list.emptyFilteredAction')}
                </Button>
              </div>
            ) : (
              <EntityList
                items={sensorsRes.data}
                getKey={(sensor) => sensor.id}
                emptyMessage={
                  view === 'prepared' ? t('list.emptyPreparedMessage') : t('list.emptyMessage')
                }
                renderItem={(sensor) => <SensorCard sensor={sensor} query={search.q ?? ''} />}
              />
            )}
            {sensorsRes.pagination && sensorsRes.pagination.totalPages > 1 && (
              <Pagination pagination={sensorsRes.pagination} />
            )}
          </div>
        )}
      </section>
    </div>
  )
}

export const Route = createFileRoute('/_protected/sensors/')({
  component: Sensors,
  validateSearch: sensorFilterSchema,
  beforeLoad: ({ search }) => {
    const legacy = legacyPreparedRedirect(search)
    if (legacy) throw redirect({ to: '/sensors', search: legacy, replace: true })
  },
  pendingComponent: pendingLoading({ key: 'sensor:list.loadingLabel' }),
  loaderDeps: ({ search }) => ({
    page: search.page,
    view: search.view,
    q: search.q,
    statuses: search.statuses,
    modelIds: search.modelIds,
    dataHealth: search.dataHealth,
    hasTree: search.hasTree,
    clusterIds: search.clusterIds,
    sort: search.sort,
    order: search.order,
  }),
  loader: ({ deps, context: { queryClient } }) => {
    prefetch(queryClient, sensorQueries.list(listParams(deps)), 'sensorQueries.list')
  },
})
