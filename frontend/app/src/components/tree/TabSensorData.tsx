import React from 'react'
import GeneralStatusCard from '../general/cards/GeneralStatusCard'
import EntitiesStatusCard from '../general/cards/EntitiesStatusCard'
import { getSensorStatusDetails } from '@/hooks/details/useDetailsForSensorStatus'
import { format } from 'date-fns'
import GeneralLink from '../general/links/GeneralLink'
import ChartSensorData from './ChartSensorData'
import { SensorStatus, Tree } from '@green-ecolution/backend-client'

interface TabSensorDataProps {
  tree: Tree
}

const TabSensorData: React.FC<TabSensorDataProps> = ({ tree }) => {
  const updatedDate = tree?.sensor?.updatedAt
    ? format(new Date(tree?.sensor?.updatedAt), 'dd.MM.yyyy')
    : 'Keine Angabe'
  const updatedTime = tree?.sensor?.latestData?.updatedAt
    ? format(new Date(tree?.sensor?.latestData?.updatedAt).getTime(), 'HH:mm')
    : 'Keine Angabe'

  const sensorStatus = tree?.sensor?.status ?? SensorStatus.SensorStatusUnknown

  return (
    <>
      <ul className="mb-5 space-y-5 md:space-y-0 md:grid md:gap-5 md:grid-cols-2 lg:grid-cols-3">
        <li>
          <EntitiesStatusCard
            statusDetails={getSensorStatusDetails(sensorStatus)}
            label="Status der Sensoren"
          />
        </li>
        <li>
          <GeneralStatusCard
            overline="Akkustand"
            value={
              tree?.sensor?.latestData?.battery
                ? `${tree?.sensor?.latestData?.battery.toFixed(2)} V`
                : 'Keine Angabe'
            }
            isLarge
            description="Ab einem Wert von 2.8 V schaltet sich die Batterie ab."
          />
        </li>
        <li>
          <GeneralStatusCard
            overline="Letzte Messung"
            value={tree?.sensor?.latestData?.updatedAt ? `${updatedTime} Uhr` : `Keine Angabe`}
            isLarge
            description={`am ${updatedDate}`}
          />
        </li>
      </ul>
      <GeneralLink
        label="Zum verknüpften Sensor"
        link={{
          to: '/sensors/$sensorId',
          params: { sensorId: String(tree?.sensor?.id) },
        }}
      />

      {tree?.sensor && <ChartSensorData sensorId={tree.sensor.id} />}
    </>
  )
}

export default TabSensorData
