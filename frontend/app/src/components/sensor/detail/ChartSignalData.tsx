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

interface ChartSignalDataProps {
  sensorId: string
}

const ChartSignalData: React.FC<ChartSignalDataProps> = ({ sensorId }) => {
  const { data: sensorDataRes } = useSuspenseQuery(sensorDataQuery(sensorId))
  const signalData = sensorDataRes
    .filter((entry) => typeof entry.signal?.rssiDbm === 'number')
    .map((entry) => ({
      name: format(new Date(entry.updatedAt), 'dd.MM.yyyy'),
      rssi: entry.signal?.rssiDbm,
    }))
    .reverse()

  if (signalData.length <= 1) return null

  return (
    <section className="mt-16">
      <h2 className="font-bold font-lato text-xl mb-4">Signalstärke im Verlaufe der Zeit</h2>
      <p className="mb-6">
        RSSI in dBm der empfangenen Messungen. Höhere (weniger negative) Werte bedeuten besseren
        Empfang.
      </p>
      <ResponsiveContainer height={400} width="100%">
        <AreaChart data={signalData} margin={{ top: 10, right: 30, left: 0, bottom: 0 }}>
          <CartesianGrid strokeDasharray="3 3" />
          <XAxis dataKey="name" />
          <YAxis />
          <Tooltip />
          <Legend />
          <Area type="monotone" dataKey="rssi" name="RSSI in dBm" stroke="#4C7741" fill="#D9E8D5" />
        </AreaChart>
      </ResponsiveContainer>
    </section>
  )
}

export default ChartSignalData
