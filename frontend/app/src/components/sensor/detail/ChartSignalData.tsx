import { useState } from 'react'
import { sensorDataQuery } from '@/api/queries'
import { keepPreviousData, useQuery } from '@tanstack/react-query'
import { format } from 'date-fns'
import { Area, AreaChart, CartesianGrid, XAxis, YAxis } from 'recharts'
import {
  Button,
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  Loading,
  type ChartConfig,
} from '@green-ecolution/ui'
import { SIGNAL_WINDOWS, windowStart, type SignalWindowKey } from './signalWindows'

const chartConfig = {
  rssi: {
    label: 'RSSI (dBm)',
    color: '#4C7741',
  },
} satisfies ChartConfig

const DAY_MS = 24 * 60 * 60 * 1000
const PER_PAGE = 5000

interface ChartSignalDataProps {
  sensorId: string
}

const ChartSignalData: React.FC<ChartSignalDataProps> = ({ sensorId }) => {
  const [selectedWindow, setSelectedWindow] = useState<SignalWindowKey>('7d')
  // eslint-disable-next-line react-hooks/purity, react-x/purity -- windowStart truncates to the hour, keeping the query key stable
  const from = windowStart(selectedWindow, Date.now())
  const {
    data: sensorDataRes,
    isPlaceholderData,
    error,
  } = useQuery({
    ...sensorDataQuery(sensorId, { from, perPage: PER_PAGE }),
    placeholderData: keepPreviousData,
  })
  if (error) throw error

  // Plot against the reading timestamp (numeric X axis) rather than a formatted
  // date string: same-day readings would otherwise collapse to one category and
  // the tooltip would stick to the first point.
  const signalData = (sensorDataRes?.data ?? [])
    .filter((entry) => typeof entry.signal?.rssiDbm === 'number')
    .map((entry) => ({
      ts: new Date(entry.updatedAt).getTime(),
      rssi: entry.signal!.rssiDbm,
    }))
    .sort((a, b) => a.ts - b.ts)

  const spanMs = signalData.length > 1 ? signalData[signalData.length - 1].ts - signalData[0].ts : 0
  const tickPattern = spanMs > 2 * DAY_MS ? 'dd.MM.' : 'HH:mm'

  return (
    <div className="mt-6 border-t border-dark-100 pt-5">
      <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
        <div>
          <p className="text-sm font-semibold">RSSI-Verlauf</p>
          <p className="text-xs text-muted-foreground">
            Höher (weniger negativ) = besserer Empfang
          </p>
        </div>
        <div role="group" aria-label="Zeitraum" className="flex items-center gap-1">
          {(Object.keys(SIGNAL_WINDOWS) as SignalWindowKey[]).map((key) => (
            <Button
              key={key}
              size="sm"
              variant={key === selectedWindow ? 'default' : 'ghost'}
              aria-pressed={key === selectedWindow}
              onClick={() => setSelectedWindow(key)}
            >
              {SIGNAL_WINDOWS[key].label}
            </Button>
          ))}
        </div>
      </div>
      {!sensorDataRes ? (
        <Loading className="h-[220px] justify-center" label="Signaldaten werden geladen" />
      ) : signalData.length <= 1 ? (
        <p className="flex h-[220px] items-center justify-center text-sm text-muted-foreground">
          Zu wenige Datenpunkte im gewählten Zeitraum.
        </p>
      ) : (
        <div
          className="transition-opacity duration-200"
          style={{ opacity: isPlaceholderData ? 0.6 : 1 }}
          aria-busy={isPlaceholderData}
        >
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
      )}
    </div>
  )
}

export default ChartSignalData
