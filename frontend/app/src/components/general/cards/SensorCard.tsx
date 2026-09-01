import type { Sensor } from '@/api/backendApi'
import { format, formatDistanceToNow } from 'date-fns'
import React from 'react'
import { useTranslation } from 'react-i18next'
import { Badge, ListCard, ListCardTitle, ListCardDescription } from '@green-ecolution/ui'
import { useSensorStatusDetails } from '@/hooks/details/useDetailsForSensorStatus'
import { useDataQualityDetails, hasQualityWarning } from '@/hooks/details/useDetailsForDataHealth'
import { useDateLocale } from '@/lib/i18n/useFormatters'
import { Link } from '@tanstack/react-router'

interface SensorCardProps {
  sensor: Sensor
}

const SensorCard: React.FC<SensorCardProps> = ({ sensor }) => {
  const { t } = useTranslation(['sensor', 'common'])
  const dateLocale = useDateLocale()
  const getSensorStatusDetails = useSensorStatusDetails()
  const getDataQualityDetails = useDataQualityDetails()
  const statusDetails = getSensorStatusDetails(sensor.status)
  const createdDate = sensor?.createdAt
    ? format(new Date(sensor?.createdAt), 'dd.MM.yyyy')
    : t('common:state.noData')
  const updatedDate = sensor?.latestData?.createdAt
    ? formatDistanceToNow(sensor?.latestData?.updatedAt, { locale: dateLocale, addSuffix: true })
    : t('common:state.noData')

  return (
    <ListCard asChild columns="1fr 2fr 1fr 1fr" className="lg:py-10">
      <Link
        to={`/sensors/$sensorId`}
        params={{
          sensorId: sensor.id,
        }}
      >
        <div className="flex flex-wrap items-center gap-2">
          <Badge variant={statusDetails.color} size="lg">
            {statusDetails.label}
          </Badge>
          {hasQualityWarning(sensor) && (
            <Badge variant={getDataQualityDetails(sensor).color}>
              {getDataQualityDetails(sensor).label}
            </Badge>
          )}
        </div>

        <div>
          <ListCardTitle className="mb-0.5">{t('card.idLabel', { id: sensor.id })}</ListCardTitle>
          <p className="text-dark-800 text-sm">{t('card.detailsLabel')}</p>
        </div>

        <ListCardDescription>
          <span className="lg:sr-only">{t('card.createdAtLabel')}&nbsp;</span>
          {createdDate}
        </ListCardDescription>

        <ListCardDescription>
          <span className="lg:sr-only">{t('card.lastUpdateLabel')}&nbsp;</span>
          {updatedDate}
        </ListCardDescription>
      </Link>
    </ListCard>
  )
}

export default SensorCard
