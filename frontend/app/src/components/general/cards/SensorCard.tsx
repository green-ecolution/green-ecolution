import type { Sensor } from '@/api/backendApi'
import { format, formatDistanceToNow } from 'date-fns'
import React from 'react'
import { Badge, ListCard, ListCardTitle, ListCardDescription } from '@green-ecolution/ui'
import { getSensorStatusDetails } from '@/hooks/details/useDetailsForSensorStatus'
import { Link } from '@tanstack/react-router'
import { de } from 'date-fns/locale'

interface SensorCardProps {
  sensor: Sensor
}

const SensorCard: React.FC<SensorCardProps> = ({ sensor }) => {
  const statusDetails = getSensorStatusDetails(sensor.status)
  const createdDate = sensor?.createdAt
    ? format(new Date(sensor?.createdAt), 'dd.MM.yyyy')
    : 'Keine Angabe'
  const updatedDate = sensor?.latestData?.createdAt
    ? formatDistanceToNow(sensor?.latestData?.updatedAt, { locale: de })
    : 'Keine Angabe'

  return (
    <ListCard asChild columns="1fr 2fr 1fr 1fr" className="lg:py-10">
      <Link
        to={`/sensors/$sensorId`}
        params={{
          sensorId: sensor.id,
        }}
      >
        <div>
          <Badge variant={statusDetails.color} size="lg">
            {statusDetails.label}
          </Badge>
        </div>

        <div>
          <ListCardTitle className="mb-0.5">ID: {sensor.id}</ListCardTitle>
          <p className="text-dark-800 text-sm">Sensor-Details</p>
        </div>

        <ListCardDescription>
          <span className="lg:sr-only">Erstellt am:&nbsp;</span>
          {createdDate}
        </ListCardDescription>

        <ListCardDescription>
          <span className="lg:sr-only">Letztes Update:&nbsp;</span>
          {updatedDate}
        </ListCardDescription>
      </Link>
    </ListCard>
  )
}

export default SensorCard
