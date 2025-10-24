import { sensorDataQuery } from '@/api/queries'
import { useSuspenseQuery } from '@tanstack/react-query'
import { format } from 'date-fns'
import {
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
  Area,
  AreaChart,
} from 'recharts'

interface ChartSensorDataProps {
  sensorId: string
}

const ChartSensorData: React.FC<ChartSensorDataProps> = ({ sensorId }) => {
  const { data: sensorDataRes } = useSuspenseQuery(sensorDataQuery(sensorId))
  const batteryData = sensorDataRes.data
    .map((entry) => ({
      name: format(new Date(entry.updatedAt), 'dd.MM.yyyy'),
      battery: entry.battery,
    }))
    .reverse()

  return (
    <>
      {sensorDataRes.data.length > 1 && (
        <section className="mt-16">
          <h2 className="font-bold font-lato text-xl mb-4">Akkulaufzeit im Verlaufe der Zeit</h2>
          <p className="mb-6">
            In diesem Abschnitt wird die Batteriewerte in Volt ausgegeben, die im System
            abgespeichert wurden.
            <br />
            Anhand dessen kann nachvollzogen werden, wie sich die Batterie im Laufe der Zeit
            entlädt.
          </p>
          <ResponsiveContainer height={400} width="100%">
            <AreaChart data={batteryData} margin={{ top: 10, right: 30, left: 0, bottom: 0 }}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="name" />
              <YAxis />
              <Tooltip />
              <Legend />
              <Area
                type="monotone"
                dataKey="battery"
                name="Batteriewerte in Volt"
                stroke="#4C7741"
                fill="#D9E8D5"
              />
            </AreaChart>
          </ResponsiveContainer>
        </section>
      )}
    </>
  )
}

export default ChartSensorData
