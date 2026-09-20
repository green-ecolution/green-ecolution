import { sensorQueries } from '@/api/queries'
import { Button, Loading } from '@green-ecolution/ui'
import Pagination from '@/components/general/Pagination'
import EntityList from '@/components/general/EntityList'
import SensorCard from '@/components/general/cards/SensorCard'
import SensorListToolbar from '@/components/sensor/list/SensorListToolbar'
import { useSensorListSearch } from '@/components/sensor/list/useSensorListSearch'
import { useQuery, keepPreviousData } from '@tanstack/react-query'
import { createFileRoute, Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { Zap } from 'lucide-react'
import { z } from 'zod'
import {
  DataHealth,
  ListSensorsOrderEnum,
  ListSensorsSortEnum,
  SensorStatus,
} from '@green-ecolution/backend-client'
import { pendingLoading, prefetch } from '@/lib/router'
import { Can } from '@/lib/auth/Can'

export const PER_PAGE = 25

const sensorFilterSchema = z.object({
  page: z.number().int().min(1).catch(1),
  q: z.string().optional().catch(undefined),
  statuses: z.array(z.enum(SensorStatus)).optional().catch(undefined),
  modelIds: z.array(z.string()).optional().catch(undefined),
  dataHealth: z.array(z.enum(DataHealth)).optional().catch(undefined),
  hasTree: z.boolean().optional().catch(undefined),
  sort: z.enum(ListSensorsSortEnum).optional().catch(undefined),
  order: z.enum(ListSensorsOrderEnum).optional().catch(undefined),
})

const listParams = (search: z.infer<typeof sensorFilterSchema>) => ({
  page: search.page,
  perPage: PER_PAGE,
  q: search.q,
  status: search.statuses,
  modelId: search.modelIds,
  dataHealth: search.dataHealth,
  hasTree: search.hasTree,
  sort: search.sort,
  order: search.order,
})

function Sensors() {
  const { t } = useTranslation('sensor')
  const search = Route.useSearch()
  const { resetFilters } = useSensorListSearch()
  const {
    data: sensorsRes,
    isPlaceholderData,
    error,
  } = useQuery({
    ...sensorQueries.list(listParams(search)),
    placeholderData: keepPreviousData,
  })
  if (error) throw error

  const isFiltered =
    (search.q ?? '').length > 0 ||
    (search.statuses?.length ?? 0) > 0 ||
    (search.modelIds?.length ?? 0) > 0 ||
    (search.dataHealth?.length ?? 0) > 0 ||
    search.hasTree !== undefined

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

      <section className="mt-8">
        <SensorListToolbar
          filteredRecords={sensorsRes?.pagination?.totalRecords ?? 0}
          totalRecords={sensorsRes?.pagination?.totalUnfiltered ?? undefined}
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
                emptyMessage={t('list.emptyMessage')}
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
  pendingComponent: pendingLoading({ key: 'sensor:list.loadingLabel' }),
  loaderDeps: ({ search }) => ({
    page: search.page,
    q: search.q,
    statuses: search.statuses,
    modelIds: search.modelIds,
    dataHealth: search.dataHealth,
    hasTree: search.hasTree,
    sort: search.sort,
    order: search.order,
  }),
  loader: ({ deps, context: { queryClient } }) => {
    prefetch(queryClient, sensorQueries.list(listParams(deps)), 'sensorQueries.list')
  },
})
