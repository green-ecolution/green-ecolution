import {
  ChartContainer,
  ChartLegend,
  ChartLegendContent,
  ChartTooltip,
  ChartTooltipContent,
  type ChartConfig,
} from '@green-ecolution/ui'
import { AreaChart, CartesianGrid, LineChart, XAxis, YAxis } from 'recharts'

interface TimeSeriesChartProps {
  config: ChartConfig
  data: Record<string, unknown>[]
  variant?: 'line' | 'area'
  children: React.ReactNode
}

/** Shared scaffold (grid, axes, tooltip, legend) for the tree detail time-series charts. */
const TimeSeriesChart = ({ config, data, variant = 'line', children }: TimeSeriesChartProps) => {
  const margin = { top: 10, right: 12, left: 0, bottom: 0 }
  const frame = [
    <CartesianGrid key="grid" vertical={false} strokeDasharray="3 3" />,
    <XAxis key="x" dataKey="name" tickLine={false} axisLine={false} tickMargin={8} />,
    <YAxis key="y" tickLine={false} axisLine={false} width={40} tickMargin={4} />,
    <ChartTooltip key="tooltip" content={<ChartTooltipContent />} />,
    <ChartLegend key="legend" content={<ChartLegendContent />} />,
  ]

  return (
    <ChartContainer config={config} className="h-[400px] w-full">
      {variant === 'area' ? (
        <AreaChart data={data} margin={margin}>
          {frame}
          {children}
        </AreaChart>
      ) : (
        <LineChart data={data} margin={margin}>
          {frame}
          {children}
        </LineChart>
      )}
    </ChartContainer>
  )
}

export default TimeSeriesChart
