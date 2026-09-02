import { sensorQueries } from '@/api/queries'
import { Button, ListCardHeader, Loading } from '@green-ecolution/ui'
import Pagination from '@/components/general/Pagination'
import EntityList from '@/components/general/EntityList'
import SensorCard from '@/components/general/cards/SensorCard'
import { useQuery, keepPreviousData } from '@tanstack/react-query'
import { createFileRoute, Link } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { Zap } from 'lucide-react'
import { z } from 'zod'
import { pendingLoading, prefetch } from '@/lib/router'
import { Can } from '@/lib/auth/Can'

export const Route = createFileRoute('/_protected/sensors/')({
  component: Sensors,
  pendingComponent: pendingLoading({ key: 'sensor:list.loadingLabel' }),
  validateSearch: z.object({
    page: z.number().int().min(1).catch(1),
  }),
  loaderDeps: ({ search: { page } }) => ({
    page,
  }),
  loader: ({ context: { queryClient }, deps: { page } }) => {
    prefetch(queryClient, sensorQueries.list({ page, perPage: 5 }), 'sensorQueries.list')
  },
})

function Sensors() {
  const { t } = useTranslation('sensor')
  const { page } = Route.useSearch()
  const {
    data: sensorsRes,
    isPlaceholderData,
    error,
  } = useQuery({
    ...sensorQueries.list({ page, perPage: 5 }),
    placeholderData: keepPreviousData,
  })
  if (error) throw error

  return (
    <div className="container mt-6">
      <div className="flex flex-col gap-4 sm:flex-row sm:items-start sm:justify-between">
        <article className="flex-1">
          <h1 className="font-lato font-bold text-3xl mb-2 lg:text-4xl xl:text-5xl">
            {t('list.title')}
          </h1>
          <p className="text-sm text-muted-foreground max-w-prose">{t('list.description')}</p>
        </article>
        <Can permission={['sensor:create']}>
          <Button asChild size="sm" className="w-full sm:w-auto sm:shrink-0">
            <Link to="/sensors/new">
              <Zap />
              {t('list.activateButton')}
            </Link>
          </Button>
        </Can>
      </div>

      <section className="mt-8">
        <ListCardHeader columns="1fr 2fr 1fr 1fr">
          <p>{t('list.statusColumn')}</p>
          <p>{t('list.nameAndLinkColumn')}</p>
          <p>{t('list.createdAtColumn')}</p>
          <p>{t('list.lastUpdateColumn')}</p>
        </ListCardHeader>

        {!sensorsRes ? (
          <Loading className="mt-10 justify-center" label={t('list.loadingLabel')} />
        ) : (
          <div
            className="transition-opacity duration-200"
            style={{ opacity: isPlaceholderData ? 0.6 : 1 }}
            aria-busy={isPlaceholderData}
          >
            <EntityList
              items={sensorsRes.data}
              getKey={(sensor) => sensor.id}
              emptyMessage={t('list.emptyMessage')}
              renderItem={(sensor) => <SensorCard sensor={sensor} />}
            />
            {sensorsRes.pagination && sensorsRes.pagination?.totalPages > 1 && (
              <Pagination pagination={sensorsRes.pagination} />
            )}
          </div>
        )}
      </section>
    </div>
  )
}
