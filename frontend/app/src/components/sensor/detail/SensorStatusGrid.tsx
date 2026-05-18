import { StatusCard } from '@green-ecolution/ui'
import { getSensorStatusDetails } from '@/hooks/details/useDetailsForSensorStatus'
import {
  formatBatteryVoltage,
  formatLastSeen,
  parseBatteryVoltage,
} from './latestDataParsing'
import type { Sensor } from '@/api/backendApi'

interface SensorStatusGridProps {
  sensor: Sensor
}

const SensorStatusGrid = ({ sensor }: SensorStatusGridProps) => {
  const status = getSensorStatusDetails(sensor.status)
  const battery = parseBatteryVoltage(sensor.latestData)
  const batteryStatus =
    battery === null ? 'default' : battery < 2.8 ? 'outline-red' : 'outline-green-dark'

  return (
    <section aria-labelledby="sensor-status-heading">
      <h2 id="sensor-status-heading" className="sr-only">
        Status
      </h2>
      <ul className="grid gap-4 md:grid-cols-3">
        <li>
          <StatusCard
            status={status.color}
            indicator="dot"
            label="Status"
            value={status.label}
            description={status.description}
          />
        </li>
        <li>
          <StatusCard
            status={batteryStatus}
            label="Akkustand"
            value={formatBatteryVoltage(sensor.latestData)}
            isLarge
            description={
              battery === null
                ? 'Noch keine Akku-Daten empfangen.'
                : 'Ab 2.8 V schaltet sich die Batterie ab.'
            }
          />
        </li>
        <li>
          <StatusCard
            label="Letztes Signal"
            value={formatLastSeen(sensor.latestData)}
            description="Letzte Datenübermittlung"
          />
        </li>
      </ul>
    </section>
  )
}

export default SensorStatusGrid
