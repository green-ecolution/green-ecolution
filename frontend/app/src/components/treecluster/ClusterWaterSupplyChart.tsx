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
import { clusterSoilMoistureQuery } from '@/api/queries'
import TimeSeriesFrame from '@/components/general/charts/TimeSeriesFrame'
import {
  timeWindowOptions,
  windowStart,
  type TimeWindowKey,
} from '@/components/general/charts/timeWindows'
import {
  toConditionRows,
  wateringEventMarkers,
} from '@/components/general/charts/soilMoistureChart'

const RANGE_KEYS = ['24h', '7d', '30d'] satisfies TimeWindowKey[]
type RangeKey = (typeof RANGE_KEYS)[number]

const BUCKET_BY_RANGE: Record<RangeKey, 'hour' | 'day'> = {
  '24h': 'hour',
  '7d': 'day',
  '30d': 'day',
}

const BUCKET_SUBTITLE: Record<'hour' | 'day', string> = {
  hour: 'Pflanzenverfügbares Wasser (Stundenmittel), schlechteste Messtiefe · Gestrichelt = Schwellen',
  day: 'Pflanzenverfügbares Wasser (Tagesmittel), schlechteste Messtiefe · Gestrichelt = Schwellen',
}

const SUPPLY_COLOR = '#0072B2'

const config = {
  supply: { label: 'Verfügbares Wasser (%)', color: SUPPLY_COLOR },
} satisfies ChartConfig

interface ClusterWaterSupplyChartProps {
  clusterId: string
  hasSensors: boolean
}

const ClusterWaterSupplyChart = ({ clusterId, hasSensors }: ClusterWaterSupplyChartProps) => {
  const [rangeKey, setRangeKey] = useState<RangeKey>('7d')
  const bucket = BUCKET_BY_RANGE[rangeKey]
  // eslint-disable-next-line react-hooks/purity, react-x/purity -- windowStart truncates to the hour, keeping the query key stable
  const from = windowStart(rangeKey, Date.now())
  const { data, isPlaceholderData, error } = useQuery({
    ...clusterSoilMoistureQuery(clusterId, { from, bucket }),
    placeholderData: keepPreviousData,
    enabled: hasSensors,
  })
  if (error) throw error

  const rows = toConditionRows(data?.condition ?? [])
  const soilUnknown = (data?.series.length ?? 0) > 0 && data?.condition.length === 0
  const markers = wateringEventMarkers(data?.wateringEvents ?? [], rows, bucket)
  const thresholds = data?.conditionThresholds

  return (
    <Card variant="outlined">
      <CardHeader className="flex flex-row flex-wrap items-start justify-between gap-2">
        <div>
          <CardTitle>Wasserversorgung</CardTitle>
          <p className="text-xs text-muted-foreground">{BUCKET_SUBTITLE[bucket]}</p>
        </div>
        <TimeRangeToggle
          options={timeWindowOptions(RANGE_KEYS)}
          value={rangeKey}
          onChange={setRangeKey}
        />
      </CardHeader>
      <CardContent>
        {!hasSensors ? (
          <p className="flex h-[260px] items-center justify-center text-center text-sm text-muted-foreground">
            Kein Baum dieser Gruppe ist mit einem Sensor ausgestattet — es liegen keine Messwerte
            vor.
          </p>
        ) : !data ? (
          <Loading className="h-[260px] justify-center" label="Messwerte werden geladen" />
        ) : soilUnknown ? (
          <p className="flex h-[260px] items-center justify-center text-center text-sm text-muted-foreground">
            Für diese Gruppe ist keine Bodenart hinterlegt — die Wasserversorgung kann nicht
            berechnet werden.
          </p>
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
              yDomain={[
                (min: number) => Math.max(0, Math.floor(min - 5)),
                (max: number) => Math.ceil(max + 5),
              ]}
            >
              <Area
                dataKey="supplyRange"
                stroke="none"
                fill={SUPPLY_COLOR}
                fillOpacity={0.15}
                connectNulls
                activeDot={false}
                tooltipType="none"
                legendType="none"
              />
              <Line
                type="monotone"
                dataKey="supply"
                stroke={SUPPLY_COLOR}
                strokeWidth={2}
                dot={false}
                activeDot={{ r: 4 }}
                connectNulls
              />
              {/* Two separate conditionals on purpose: Recharts does not reliably render children wrapped in a Fragment. */}
              {thresholds && (
                <ReferenceLine
                  y={thresholds.moderate}
                  stroke="#747474"
                  strokeDasharray="4 4"
                  ifOverflow="extendDomain"
                  label={{ value: 'mäßig', position: 'insideBottomLeft', fontSize: 11 }}
                />
              )}
              {thresholds && (
                <ReferenceLine
                  y={thresholds.critical}
                  stroke="#747474"
                  strokeDasharray="4 4"
                  ifOverflow="extendDomain"
                  label={{ value: 'kritisch', position: 'insideBottomLeft', fontSize: 11 }}
                />
              )}
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

export default ClusterWaterSupplyChart
