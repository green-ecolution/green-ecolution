import { sensorDataQuery } from '@/api/queries'
import { useSuspenseQuery } from '@tanstack/react-query'
import { format } from 'date-fns'
import { Area } from 'recharts'
import { type ChartConfig } from '@green-ecolution/ui'
import TimeSeriesChart from './TimeSeriesChart'

const chartConfig = {
  battery: {
    label: 'Batteriewerte in Volt',
    color: '#4C7741',
  },
} satisfies ChartConfig

interface ChartSensorDataProps {
  sensorId: string
}

const ChartSensorData: React.FC<ChartSensorDataProps> = ({ sensorId }) => {
  const { data: sensorDataRes } = useSuspenseQuery(sensorDataQuery(sensorId))
  const batteryData = sensorDataRes
    .map((entry) => ({
      name: format(new Date(entry.updatedAt), 'dd.MM.yyyy'),
      battery: (entry.data as Record<string, unknown>).battery,
    }))
    .reverse()

  if (sensorDataRes.length <= 1) return null

  return (
    <section className="mt-16">
      <h2 className="font-bold font-lato text-xl mb-4">Akkulaufzeit im Verlaufe der Zeit</h2>
      <p className="mb-6">
        In diesem Abschnitt wird die Batteriewerte in Volt ausgegeben, die im System abgespeichert
        wurden.
        <br />
        Anhand dessen kann nachvollzogen werden, wie sich die Batterie im Laufe der Zeit entlädt.
      </p>
      <TimeSeriesChart variant="area" config={chartConfig} data={batteryData}>
        <Area
          type="monotone"
          dataKey="battery"
          stroke="var(--color-battery)"
          strokeWidth={2}
          fill="var(--color-battery)"
          fillOpacity={0.15}
        />
      </TimeSeriesChart>
    </section>
  )
}

export default ChartSensorData
