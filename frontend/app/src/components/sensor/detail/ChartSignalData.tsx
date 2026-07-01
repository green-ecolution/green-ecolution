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

const DAY_MS = 24 * 60 * 60 * 1000

interface ChartSignalDataProps {
  sensorId: string
}

const ChartSignalData: React.FC<ChartSignalDataProps> = ({ sensorId }) => {
  const { data: sensorDataRes } = useSuspenseQuery(sensorDataQuery(sensorId))
  // Plot against the reading timestamp (numeric X axis) rather than a formatted
  // date string: same-day readings would otherwise collapse to one category and
  // the tooltip would stick to the first point.
  const signalData = sensorDataRes
    .filter((entry) => typeof entry.signal?.rssiDbm === 'number')
    .map((entry) => ({
      ts: new Date(entry.updatedAt).getTime(),
      rssi: entry.signal!.rssiDbm,
    }))
    .sort((a, b) => a.ts - b.ts)

  if (signalData.length <= 1) return null

  const spanMs = signalData[signalData.length - 1].ts - signalData[0].ts
  const tickPattern = spanMs > 2 * DAY_MS ? 'dd.MM.' : 'HH:mm'

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
          <XAxis
            dataKey="ts"
            type="number"
            scale="time"
            domain={['dataMin', 'dataMax']}
            tickLine={false}
            axisLine={false}
            tickMargin={8}
            minTickGap={40}
            tickFormatter={(value) => format(new Date(value as number), tickPattern)}
          />
          <YAxis
            tickLine={false}
            axisLine={false}
            width={40}
            tickMargin={4}
            domain={['dataMin - 2', 'dataMax + 2']}
          />
          <ChartTooltip
            content={
              <ChartTooltipContent
                labelFormatter={(_, payload) => {
                  const point = payload?.[0] as { payload?: { ts?: number } } | undefined
                  const ts = point?.payload?.ts
                  return ts ? format(new Date(ts), 'dd.MM.yyyy HH:mm') : ''
                }}
              />
            }
          />
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
