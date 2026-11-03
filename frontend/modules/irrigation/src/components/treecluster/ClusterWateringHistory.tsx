import { useQuery } from '@tanstack/react-query'
import { Link } from '@tanstack/react-router'
import { format, formatDistanceToNow } from 'date-fns'
import { ChevronRight, Droplet } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Card, CardContent, CardHeader, CardTitle, Loading } from '@green-ecolution/ui'
import { clusterQueries } from '../../api/queries'
import { windowStart } from '@/components/general/charts/timeWindows'
import { useDateLocale, useNumberFormatter } from '@/lib/i18n/useFormatters'

const MAX_EVENTS = 6

interface ClusterWateringHistoryProps {
  clusterId: string
}

const ClusterWateringHistory = ({ clusterId }: ClusterWateringHistoryProps) => {
  const { t } = useTranslation('treecluster')
  const dateLocale = useDateLocale()
  const numberFormatter = useNumberFormatter()
  // Same params as ClusterSoilMoistureChart's initial state ('7d' / bucket 'day')
  // so the two components share one TanStack Query cache entry on first paint.
  // eslint-disable-next-line react-hooks/purity, react-x/purity -- windowStart truncates to the hour, keeping the query key stable
  const from = windowStart('7d', Date.now())
  const { data, isPending, error } = useQuery(
    clusterQueries.soilMoisture(clusterId, { from, bucket: 'day' }),
  )

  // Backend guarantees newest-first order (ORDER BY wp.date DESC), so no client-side sort is needed.
  const events = (data?.wateringEvents ?? []).slice(0, MAX_EVENTS)

  return (
    <Card variant="outlined">
      <CardHeader>
        <CardTitle>{t('wateringHistory.title')}</CardTitle>
      </CardHeader>
      <CardContent>
        {isPending ? (
          <Loading className="justify-center py-6" label={t('wateringHistory.loadingLabel')} />
        ) : error ? (
          <p className="text-sm text-muted-foreground">{t('wateringHistory.errorMessage')}</p>
        ) : events.length === 0 ? (
          <p className="text-sm text-muted-foreground">{t('wateringHistory.emptyMessage')}</p>
        ) : (
          <ol>
            {events.map((event, index) => (
              <li key={event.wateringPlanId} className="relative">
                {events.length > 1 && (
                  <span
                    aria-hidden
                    className={`absolute left-[7.5px] w-px bg-dark-100 ${index === 0 ? 'top-1/2' : 'top-0'} ${index === events.length - 1 ? 'bottom-1/2' : 'bottom-0'}`}
                  />
                )}
                <Link
                  to="/plans/$wateringPlanId"
                  params={{ wateringPlanId: event.wateringPlanId }}
                  className="group relative flex items-center gap-3 rounded-sm py-2 text-sm focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-green-dark"
                >
                  <span className="flex size-4 shrink-0 items-center justify-center bg-card">
                    <Droplet className="size-3.5 fill-green-dark/15 text-green-dark" aria-hidden />
                  </span>
                  <span className="min-w-0 flex-1 truncate">
                    <span className="font-semibold tabular-nums group-hover:underline">
                      {format(event.date, 'dd.MM.yyyy', { locale: dateLocale })}
                    </span>
                    <span className="text-muted-foreground">
                      {' · '}
                      {formatDistanceToNow(event.date, { locale: dateLocale, addSuffix: true })}
                    </span>
                  </span>
                  <span className="shrink-0 tabular-nums">
                    {numberFormatter.format(event.consumedWaterLiters)} L
                  </span>
                  <ChevronRight
                    className="size-4 shrink-0 text-dark-400 transition-transform duration-quick group-hover:translate-x-0.5 motion-reduce:transition-none"
                    aria-hidden
                  />
                  <span className="sr-only">{t('wateringHistory.viewLink')}</span>
                </Link>
              </li>
            ))}
          </ol>
        )}
      </CardContent>
    </Card>
  )
}

export default ClusterWateringHistory
