import { useQuery } from '@tanstack/react-query'
import { Link } from '@tanstack/react-router'
import { format } from 'date-fns'
import { useTranslation } from 'react-i18next'
import { Card, CardContent, CardHeader, CardTitle, Loading } from '@green-ecolution/ui'
import { clusterQueries } from '@/api/queries'
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
          <ul className="flex flex-col gap-y-3">
            {events.map((event) => (
              <li
                key={event.wateringPlanId}
                className="flex flex-wrap items-center justify-between gap-x-4 gap-y-1 rounded-lg border border-dark-50 bg-white p-4"
              >
                <span className="font-bold">
                  {format(event.date, 'dd.MM.yyyy', { locale: dateLocale })}
                </span>
                <Link
                  to="/watering-plans/$wateringPlanId"
                  params={{ wateringPlanId: event.wateringPlanId }}
                  className="text-sm text-green-dark hover:underline"
                >
                  {t('wateringHistory.viewLink')}
                </Link>
                <span className="tabular-nums">
                  {numberFormatter.format(event.consumedWaterLiters)} L
                </span>
              </li>
            ))}
          </ul>
        )}
      </CardContent>
    </Card>
  )
}

export default ClusterWateringHistory
