import { useState } from 'react'
import { sensorDataQuery } from '@/api/queries'
import { keepPreviousData, useQuery } from '@tanstack/react-query'
import { Area } from 'recharts'
import { Loading, TimeRangeToggle, type ChartConfig } from '@green-ecolution/ui'
import TimeSeriesFrame from '@/components/general/charts/TimeSeriesFrame'
import {
  timeWindowOptions,
  windowStart,
  type TimeWindowKey,
} from '@/components/general/charts/timeWindows'

const chartConfig = {
  rssi: {
    label: 'RSSI (dBm)',
    color: '#4C7741',
  },
} satisfies ChartConfig

const PER_PAGE = 5000

interface ChartSignalDataProps {
  sensorId: string
}

const ChartSignalData: React.FC<ChartSignalDataProps> = ({ sensorId }) => {
  const [selectedWindow, setSelectedWindow] = useState<TimeWindowKey>('7d')
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

  return (
    <div className="mt-6 border-t border-dark-100 pt-5">
      <div className="mb-3 flex flex-wrap items-center justify-between gap-2">
        <div>
          <p className="text-sm font-semibold">RSSI-Verlauf</p>
          <p className="text-xs text-muted-foreground">
            Höher (weniger negativ) = besserer Empfang
          </p>
        </div>
        <TimeRangeToggle
          options={timeWindowOptions(['24h', '7d', '30d', 'all'])}
          value={selectedWindow}
          onChange={setSelectedWindow}
        />
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
          <TimeSeriesFrame
            config={chartConfig}
            data={signalData}
            yDomain={['dataMin - 2', 'dataMax + 2']}
          >
            <defs>
              <linearGradient id="fillRssi" x1="0" y1="0" x2="0" y2="1">
                <stop offset="5%" stopColor="var(--color-rssi)" stopOpacity={0.25} />
                <stop offset="95%" stopColor="var(--color-rssi)" stopOpacity={0.03} />
              </linearGradient>
            </defs>
            <Area
              type="monotone"
              dataKey="rssi"
              stroke="var(--color-rssi)"
              strokeWidth={2}
              fill="url(#fillRssi)"
              dot={false}
              activeDot={{ r: 4 }}
            />
          </TimeSeriesFrame>
        </div>
      )}
    </div>
  )
}

export default ChartSignalData
