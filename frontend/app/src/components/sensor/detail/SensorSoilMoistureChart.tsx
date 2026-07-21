import { useState } from 'react'
import { useQuery, keepPreviousData } from '@tanstack/react-query'
import { format } from 'date-fns'
import { Area, Line, ReferenceLine } from 'recharts'
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Loading,
  TimeRangeToggle,
  type ChartConfig,
} from '@green-ecolution/ui'
import { sensorModelIdQuery, sensorSoilMoistureQuery } from '@/api/queries'
import TimeSeriesFrame from '@/components/general/charts/TimeSeriesFrame'
import {
  timeWindowOptions,
  windowStart,
  type TimeWindowKey,
} from '@/components/general/charts/timeWindows'
import {
  depthColor,
  toChartRows,
  wateringEventMarkers,
} from '@/components/general/charts/soilMoistureChart'
import type { Sensor } from '@/api/backendApi'

const RANGE_KEYS = ['24h', '7d', '30d'] satisfies TimeWindowKey[]
type RangeKey = (typeof RANGE_KEYS)[number]

const BUCKET_BY_RANGE: Record<RangeKey, 'hour' | 'day'> = {
  '24h': 'hour',
  '7d': 'day',
  '30d': 'day',
}

const BUCKET_SUBTITLE: Record<'hour' | 'day', string> = {
  hour: 'Stundenmittel · Band = Min–Max · Gestrichelt = kritische Schwelle je Tiefe',
  day: 'Tagesmittel · Band = Min–Max · Gestrichelt = kritische Schwelle je Tiefe',
}

interface SensorSoilMoistureChartProps {
  sensor: Sensor
}

const SensorSoilMoistureChart = ({ sensor }: SensorSoilMoistureChartProps) => {
  const [rangeKey, setRangeKey] = useState<RangeKey>('7d')
  const bucket = BUCKET_BY_RANGE[rangeKey]
  // eslint-disable-next-line react-hooks/purity, react-x/purity -- windowStart truncates to the hour, keeping the query key stable
  const from = windowStart(rangeKey, Date.now())
  const { data: model } = useQuery(sensorModelIdQuery(sensor.model.id))
  const hasSoilMoisture = (model?.abilities ?? []).some((a) => a.ability === 'soil_moisture')
  const { data, isPlaceholderData, error } = useQuery({
    ...sensorSoilMoistureQuery(sensor.id, { from, bucket }),
    placeholderData: keepPreviousData,
    enabled: hasSoilMoisture,
  })
  if (error) throw error
  if (!hasSoilMoisture) return null

  const rows = toChartRows(data?.series ?? [])
  const depths = (data?.series ?? []).map((s) => s.depthCm)
  const config = Object.fromEntries(
    depths.map((depth, index) => [
      `mean_${depth}`,
      { label: `${depth} cm Tiefe`, color: depthColor(depth, index) },
    ]),
  ) satisfies ChartConfig
  const criticalByDepth = new Map((data?.thresholds ?? []).map((t) => [t.depthCm, t.critical]))
  const markers = wateringEventMarkers(data?.wateringEvents ?? [], rows, bucket)

  return (
    <Card variant="outlined">
      <CardHeader className="flex flex-row flex-wrap items-start justify-between gap-2">
        <div>
          <CardTitle>Bodenfeuchte-Verlauf</CardTitle>
          <p className="text-xs text-muted-foreground">{BUCKET_SUBTITLE[bucket]}</p>
        </div>
        <TimeRangeToggle
          options={timeWindowOptions(RANGE_KEYS)}
          value={rangeKey}
          onChange={setRangeKey}
        />
      </CardHeader>
      <CardContent>
        {!data ? (
          <Loading className="h-[260px] justify-center" label="Messwerte werden geladen" />
        ) : rows.length <= 1 ? (
          <p className="flex h-[260px] items-center justify-center text-sm text-muted-foreground">
            Zu wenige Datenpunkte im gewählten Zeitraum.
          </p>
        ) : (
          <div
            className="transition-opacity duration-200"
            style={{ opacity: isPlaceholderData ? 0.6 : 1 }}
            aria-busy={isPlaceholderData}
          >
            <TimeSeriesFrame
              config={config}
              data={rows}
              className="h-[260px] w-full"
              yDomain={[(min: number) => Math.floor(min - 1), (max: number) => Math.ceil(max + 1)]}
              legend
            >
              {depths.map((depth, index) => (
                <Area
                  key={`range_${depth}`}
                  dataKey={`range_${depth}`}
                  stroke="none"
                  fill={depthColor(depth, index)}
                  fillOpacity={0.15}
                  connectNulls
                  activeDot={false}
                  tooltipType="none"
                  legendType="none"
                />
              ))}
              {depths.map((depth, index) => (
                <Line
                  key={`mean_${depth}`}
                  type="monotone"
                  dataKey={`mean_${depth}`}
                  stroke={depthColor(depth, index)}
                  strokeWidth={2}
                  dot={false}
                  activeDot={{ r: 4 }}
                  connectNulls
                />
              ))}
              {depths.map((depth, index) => {
                const critical = criticalByDepth.get(depth)
                return critical != null ? (
                  <ReferenceLine
                    key={`critical_${depth}`}
                    y={critical}
                    stroke={depthColor(depth, index)}
                    strokeDasharray="4 4"
                    ifOverflow="extendDomain"
                  />
                ) : null
              })}
              {markers.map((event) => (
                <ReferenceLine
                  key={event.wateringPlanId}
                  x={event.ts}
                  stroke="#747474"
                  strokeDasharray="4 4"
                  label={{
                    value: `Bewässert am ${format(event.date, 'dd.MM.')}`,
                    angle: -90,
                    position: 'insideTopRight',
                    fontSize: 11,
                  }}
                />
              ))}
            </TimeSeriesFrame>
          </div>
        )}
      </CardContent>
    </Card>
  )
}

export default SensorSoilMoistureChart
