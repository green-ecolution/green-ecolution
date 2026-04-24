import React from 'react'
import { StatusCard } from '@green-ecolution/ui'
import { getSensorStatusDetails } from '@/hooks/details/useDetailsForSensorStatus'
import { format } from 'date-fns'
import GeneralLink from '../general/links/GeneralLink'
import ChartSensorData from './ChartSensorData'
import { SensorStatus, Tree } from '@green-ecolution/backend-client'
import { useQuery } from '@tanstack/react-query'
import { sensorIdQuery } from '@/api/queries'

interface TabSensorDataProps {
  tree: Tree
}

const TabSensorData: React.FC<TabSensorDataProps> = ({ tree }) => {
  const { data: sensor } = useQuery({
    ...sensorIdQuery(tree.sensorId!),
    enabled: !!tree.sensorId,
  })

  const updatedDate = sensor?.updatedAt
    ? format(new Date(sensor.updatedAt), 'dd.MM.yyyy')
    : 'Keine Angabe'
  const updatedTime = sensor?.latestData?.updatedAt
    ? format(new Date(sensor.latestData.updatedAt).getTime(), 'HH:mm')
    : 'Keine Angabe'

  const sensorStatusDetails = getSensorStatusDetails(
    sensor?.status ?? SensorStatus.SensorStatusUnknown,
  )

  return (
    <>
      <ul className="mb-5 flex flex-col gap-y-5 md:grid md:gap-5 md:grid-cols-2 lg:grid-cols-3">
        <li>
          <StatusCard
            status={sensorStatusDetails.color}
            indicator="dot"
            label="Status der Sensoren"
            value={sensorStatusDetails.label}
            description={sensorStatusDetails.description}
          />
        </li>
        <li>
          <StatusCard
            label="Akkustand"
            value={
              sensor?.latestData?.battery
                ? `${sensor.latestData.battery.toFixed(2)} V`
                : 'Keine Angabe'
            }
            isLarge
            description="Ab einem Wert von 2.8 V schaltet sich die Batterie ab."
          />
        </li>
        <li>
          <StatusCard
            label="Letzte Messung"
            value={sensor?.latestData?.updatedAt ? `${updatedTime} Uhr` : 'Keine Angabe'}
            isLarge
            description={`am ${updatedDate}`}
          />
        </li>
      </ul>
      <GeneralLink
        label="Zum verknüpften Sensor"
        link={{
          to: '/sensors/$sensorId',
          params: { sensorId: String(tree.sensorId) },
        }}
      />

      {tree.sensorId && <ChartSensorData sensorId={tree.sensorId} />}
    </>
  )
}

export default TabSensorData
