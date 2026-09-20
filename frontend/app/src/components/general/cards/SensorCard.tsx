import { CalendarDays, FolderClosed, Radio, TreeDeciduous } from 'lucide-react'
import { format, formatDistanceToNow } from 'date-fns'
import type { Sensor } from '@/api/backendApi'
import { Link } from '@tanstack/react-router'
import React from 'react'
import { useTranslation } from 'react-i18next'
import { Badge, ListCard } from '@green-ecolution/ui'
import { useSensorStatusDetails } from '@/hooks/details/useDetailsForSensorStatus'
import { useDataQualityDetails, hasQualityWarning } from '@/hooks/details/useDetailsForDataHealth'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { highlightMatch } from '@/lib/highlightMatch'

interface SensorCardProps {
  sensor: Sensor
  query?: string
}

const SensorCard: React.FC<SensorCardProps> = ({ sensor, query = '' }) => {
  const { t } = useTranslation(['sensor', 'common'])
  const dateLocale = useDateLocale()
  const getSensorStatusDetails = useSensorStatusDetails()
  const getDataQualityDetails = useDataQualityDetails()
  const statusDetails = getSensorStatusDetails(sensor.status)
  const createdDate = sensor.createdAt
    ? format(new Date(sensor.createdAt), 'dd.MM.yyyy', { locale: dateLocale })
    : t('common:state.noData')
  const lastReading = sensor.latestData?.updatedAt
    ? formatDistanceToNow(sensor.latestData.updatedAt, { locale: dateLocale, addSuffix: true })
    : t('common:state.noData')

  return (
    <ListCard asChild className="@container">
      <Link to="/sensors/$sensorId" params={{ sensorId: sensor.id }}>
        <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
          <span className="flex items-center gap-2.5">
            <span
              aria-hidden
              title={statusDetails.label}
              className="size-2.5 shrink-0 rounded-full"
              style={{ backgroundColor: statusDetails.colorHex }}
            />
            <span className="font-lato text-lg font-bold tabular-nums">
              <span className="sr-only">
                {t('card.statusSrLabel')}
                {statusDetails.label}
              </span>
              <span className="sr-only">{t('card.idSrLabel')}</span>
              {highlightMatch(sensor.id, query)}
            </span>
          </span>
          <span className="text-base font-medium text-dark-800">
            {highlightMatch(sensor.model?.name ?? t('card.modelUnknown'), query)}
          </span>
          {hasQualityWarning(sensor) && (
            <Badge variant={getDataQualityDetails(sensor).color}>
              {getDataQualityDetails(sensor).label}
            </Badge>
          )}
        </div>

        <div className="flex flex-wrap items-center gap-x-6 gap-y-1 text-sm text-dark-600">
          <span className="flex items-center gap-2">
            <TreeDeciduous aria-hidden className="size-4" />
            <span className="sr-only">{t('card.treeSrLabel')}</span>
            {sensor.linkedTreeId ? t('card.treeLinked') : t('card.treeUnlinked')}
          </span>
          {sensor.linkedClusterName && (
            <span className="flex items-center gap-2">
              <FolderClosed aria-hidden className="size-4" />
              <span className="sr-only">{t('card.clusterSrLabel')}</span>
              {highlightMatch(sensor.linkedClusterName, query)}
            </span>
          )}
          <span className="flex items-center gap-2">
            <Radio aria-hidden className="size-4" />
            <span className="sr-only">{t('card.lastUpdateLabel')}</span>
            {lastReading}
          </span>
          {/* Secondary fact, dropped as the card narrows so the row never wraps. */}
          <span className="hidden items-center gap-2 @min-[30rem]:flex">
            <CalendarDays aria-hidden className="size-4" />
            <span className="sr-only">{t('card.createdAtLabel')}</span>
            {createdDate}
          </span>
        </div>
      </Link>
    </ListCard>
  )
}

export default SensorCard
