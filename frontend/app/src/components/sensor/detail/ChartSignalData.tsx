import { sensorDataQuery } from '@/api/queries'
import { useSuspenseQuery } from '@tanstack/react-query'
import { format } from 'date-fns'
import { Area, AreaChart, CartesianGrid, XAxis, YAxis } from 'recharts'
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from '@green-ecolution/ui'

const chartConfig = {
  rssi: {
    label: 'RSSI (dBm)',
    color: '#4C7741',
  },
} satisfies ChartConfig

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
    <div className="mt-6 border-t border-dark-100 pt-5">
      <div className="mb-3 flex items-baseline justify-between gap-2">
        <p className="text-sm font-semibold">RSSI-Verlauf</p>
        <p className="text-xs text-muted-foreground">Höher (weniger negativ) = besserer Empfang</p>
      </div>
      <ChartContainer config={chartConfig} className="h-[220px] w-full">
        <AreaChart data={signalData} margin={{ top: 8, right: 12, left: 0, bottom: 0 }}>
          <defs>
            <linearGradient id="fillRssi" x1="0" y1="0" x2="0" y2="1">
              <stop offset="5%" stopColor="var(--color-rssi)" stopOpacity={0.25} />
              <stop offset="95%" stopColor="var(--color-rssi)" stopOpacity={0.03} />
            </linearGradient>
          </defs>
          <CartesianGrid vertical={false} strokeDasharray="3 3" />
          <XAxis dataKey="name" tickLine={false} axisLine={false} tickMargin={8} minTickGap={24} />
          <YAxis
            tickLine={false}
            axisLine={false}
            width={40}
            tickMargin={4}
            domain={['dataMin - 2', 'dataMax + 2']}
          />
          <ChartTooltip content={<ChartTooltipContent />} />
          <Area
            type="monotone"
            dataKey="rssi"
            stroke="var(--color-rssi)"
            strokeWidth={2}
            fill="url(#fillRssi)"
            dot={false}
            activeDot={{ r: 4 }}
          />
        </AreaChart>
      </ChartContainer>
    </div>
  )
}

export default ChartSignalData
