import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  DetailedList,
  SignalBars,
} from '@green-ecolution/ui'
import { Link } from '@tanstack/react-router'
import type { Tree } from '@/api/backendApi'
import {
  parseSignal,
  signalBarsFromRssi,
  signalLevelFromRssi,
  SIGNAL_LEVEL_LABEL,
  SIGNAL_LEVEL_TEXT_COLOR,
} from '@/components/sensor/detail/signalParsing'
import { formatBatteryVoltage, formatLastSeen } from '@/components/sensor/detail/latestDataParsing'

interface ClusterSensorCardProps {
  trees: Tree[]
}

const SensorTreeRow = ({ tree }: { tree: Tree }) => {
  const sensor = tree.sensor
  if (!sensor) return null

  const signal = parseSignal(sensor.latestData)

  return (
    <Link
      to="/sensors/$sensorId"
      params={{ sensorId: sensor.id }}
      className="block rounded-lg border border-dark-50 bg-white p-4 transition-colors hover:border-green-dark"
    >
      <p className="mb-2 font-bold">
        Sensor-Baum: {tree.species} · {tree.number}
      </p>
      <DetailedList
        columns={1}
        details={[
          {
            label: 'Signal',
            value: signal ? (
              <span
                className={`flex items-center gap-2 ${SIGNAL_LEVEL_TEXT_COLOR[signalLevelFromRssi(signal.rssiDbm)]}`}
              >
                <SignalBars filled={signalBarsFromRssi(signal.rssiDbm)} />
                {SIGNAL_LEVEL_LABEL[signalLevelFromRssi(signal.rssiDbm)]}
              </span>
            ) : (
              'Keine Daten'
            ),
          },
          { label: 'Batterie', value: formatBatteryVoltage(sensor.latestData) },
          { label: 'Letzte Übertragung', value: formatLastSeen(sensor.latestData) },
        ]}
      />
    </Link>
  )
}

const ClusterSensorCard = ({ trees }: ClusterSensorCardProps) => {
  const treesWithSensor = trees.filter((tree) => tree.sensor)

  return (
    <Card variant="outlined">
      <CardHeader>
        <CardTitle>Sensorik</CardTitle>
      </CardHeader>
      <CardContent>
        {treesWithSensor.length === 0 ? (
          <p className="text-muted-foreground">
            Kein Baum dieser Gruppe ist mit einem Sensor ausgestattet.
          </p>
        ) : (
          <>
            <div className="flex flex-col gap-y-3">
              {treesWithSensor.map((tree) => (
                <SensorTreeRow key={tree.id} tree={tree} />
              ))}
            </div>
            <p className="mt-4 text-right text-sm text-muted-foreground tabular-nums">
              {treesWithSensor.length} von {trees.length} {trees.length === 1 ? 'Baum' : 'Bäumen'}{' '}
              mit Sensor
            </p>
          </>
        )}
      </CardContent>
    </Card>
  )
}

export default ClusterSensorCard
