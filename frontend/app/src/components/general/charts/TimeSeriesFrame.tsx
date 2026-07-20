import { format } from 'date-fns'
import { CartesianGrid, ComposedChart, XAxis, YAxis } from 'recharts'
import {
  ChartContainer,
  ChartLegend,
  ChartLegendContent,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from '@green-ecolution/ui'

const DAY_MS = 24 * 60 * 60 * 1000

interface TimeSeriesFrameProps {
  config: ChartConfig
  /** Rows must carry `ts` (epoch ms). */
  data: Record<string, unknown>[]
  yDomain?: [string | number, string | number]
  className?: string
  legend?: boolean
  children: React.ReactNode
}

/** Shared scaffold for time-series charts: numeric time axis, grid, tooltip. */
const TimeSeriesFrame = ({
  config,
  data,
  yDomain = ['auto', 'auto'],
  className = 'h-[220px] w-full',
  legend = false,
  children,
}: TimeSeriesFrameProps) => {
  const ts = data.map((row) => row.ts as number)
  const spanMs = ts.length > 1 ? ts[ts.length - 1] - ts[0] : 0
  const tickPattern = spanMs > 2 * DAY_MS ? 'dd.MM.' : 'HH:mm'

  return (
    <ChartContainer config={config} className={className}>
      <ComposedChart data={data} margin={{ top: 8, right: 12, left: 0, bottom: 0 }}>
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
        <YAxis tickLine={false} axisLine={false} width={40} tickMargin={4} domain={yDomain} />
        <ChartTooltip
          content={
            <ChartTooltipContent
              labelFormatter={(_, payload) => {
                const point = payload?.[0] as { payload?: { ts?: number } } | undefined
                const pointTs = point?.payload?.ts
                return pointTs ? format(new Date(pointTs), 'dd.MM.yyyy HH:mm') : ''
              }}
            />
          }
        />
        {legend && <ChartLegend content={<ChartLegendContent />} />}
        {children}
      </ComposedChart>
    </ChartContainer>
  )
}

export default TimeSeriesFrame
