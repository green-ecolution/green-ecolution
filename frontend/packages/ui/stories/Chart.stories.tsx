import type { Meta, StoryObj } from '@storybook/react-vite'
import { useEffect, useState } from 'react'
import {
  Line,
  LineChart,
  CartesianGrid,
  XAxis,
  YAxis,
  Area,
  AreaChart,
  Bar,
  BarChart,
} from 'recharts'
import {
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  ChartLegend,
  ChartLegendContent,
  type ChartConfig,
} from '../src/components/ui/chart'

const meta: Meta = {
  title: 'UI/Chart',
  tags: ['autodocs'],
  parameters: {
    layout: 'padded',
  },
}

export default meta
type Story = StoryObj<typeof meta>

// Sample data for memory usage over time
const memoryData = [
  { time: '10:00', memory: 42, heap: 38 },
  { time: '10:02', memory: 45, heap: 41 },
  { time: '10:04', memory: 48, heap: 44 },
  { time: '10:06', memory: 43, heap: 39 },
  { time: '10:08', memory: 52, heap: 48 },
  { time: '10:10', memory: 49, heap: 45 },
  { time: '10:12', memory: 55, heap: 51 },
  { time: '10:14', memory: 51, heap: 47 },
]

// Sample data for service status
const serviceData = [
  { name: 'Datenbank', responseTime: 5 },
  { name: 'S3', responseTime: 42 },
  { name: 'Routing', responseTime: 125 },
  { name: 'Auth', responseTime: 68 },
  { name: 'MQTT', responseTime: 12 },
]

// Sample data for entity counts
const entityData = [
  { name: 'Bäume', count: 1234 },
  { name: 'Sensoren', count: 56 },
  { name: 'Fahrzeuge', count: 12 },
  { name: 'Gruppen', count: 45 },
  { name: 'Pläne', count: 8 },
]

export const LineChartDefault: Story = {
  render: () => {
    const chartConfig: ChartConfig = {
      memory: {
        label: 'Speicher (MB)',
        color: 'hsl(142, 76%, 36%)',
      },
    }

    return (
      <div className="w-full max-w-2xl">
        <h3 className="text-lg font-semibold mb-4">Speicherverbrauch</h3>
        <ChartContainer config={chartConfig} className="h-[300px] w-full">
          <LineChart data={memoryData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="time" />
            <YAxis />
            <ChartTooltip content={<ChartTooltipContent />} />
            <Line
              type="monotone"
              dataKey="memory"
              stroke="var(--color-memory)"
              strokeWidth={2}
              dot={false}
            />
          </LineChart>
        </ChartContainer>
      </div>
    )
  },
}

export const LineChartMultipleSeries: Story = {
  render: () => {
    const chartConfig: ChartConfig = {
      memory: {
        label: 'Gesamt (MB)',
        color: 'hsl(142, 76%, 36%)',
      },
      heap: {
        label: 'Heap (MB)',
        color: 'hsl(217, 91%, 60%)',
      },
    }

    return (
      <div className="w-full max-w-2xl">
        <h3 className="text-lg font-semibold mb-4">Speicherverbrauch (Detail)</h3>
        <ChartContainer config={chartConfig} className="h-[300px] w-full">
          <LineChart data={memoryData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="time" />
            <YAxis />
            <ChartTooltip content={<ChartTooltipContent />} />
            <ChartLegend content={<ChartLegendContent />} />
            <Line
              type="monotone"
              dataKey="memory"
              stroke="var(--color-memory)"
              strokeWidth={2}
              dot={false}
            />
            <Line
              type="monotone"
              dataKey="heap"
              stroke="var(--color-heap)"
              strokeWidth={2}
              dot={false}
            />
          </LineChart>
        </ChartContainer>
      </div>
    )
  },
}

export const AreaChartMemory: Story = {
  render: () => {
    const chartConfig: ChartConfig = {
      memory: {
        label: 'Speicher (MB)',
        color: 'hsl(142, 76%, 36%)',
      },
    }

    return (
      <div className="w-full max-w-2xl">
        <h3 className="text-lg font-semibold mb-4">Speicherverbrauch (Area)</h3>
        <ChartContainer config={chartConfig} className="h-[300px] w-full">
          <AreaChart data={memoryData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="time" />
            <YAxis />
            <ChartTooltip content={<ChartTooltipContent />} />
            <Area
              type="monotone"
              dataKey="memory"
              stroke="var(--color-memory)"
              fill="var(--color-memory)"
              fillOpacity={0.3}
            />
          </AreaChart>
        </ChartContainer>
      </div>
    )
  },
}

export const BarChartServices: Story = {
  render: () => {
    const chartConfig: ChartConfig = {
      responseTime: {
        label: 'Antwortzeit (ms)',
        color: 'hsl(217, 91%, 60%)',
      },
    }

    return (
      <div className="w-full max-w-2xl">
        <h3 className="text-lg font-semibold mb-4">Service Antwortzeiten</h3>
        <ChartContainer config={chartConfig} className="h-[300px] w-full">
          <BarChart data={serviceData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="name" />
            <YAxis />
            <ChartTooltip content={<ChartTooltipContent />} />
            <Bar dataKey="responseTime" fill="var(--color-responseTime)" radius={[4, 4, 0, 0]} />
          </BarChart>
        </ChartContainer>
      </div>
    )
  },
}

export const BarChartEntities: Story = {
  render: () => {
    const chartConfig: ChartConfig = {
      count: {
        label: 'Anzahl',
        color: 'hsl(142, 76%, 36%)',
      },
    }

    return (
      <div className="w-full max-w-2xl">
        <h3 className="text-lg font-semibold mb-4">Daten-Statistiken</h3>
        <ChartContainer config={chartConfig} className="h-[300px] w-full">
          <BarChart data={entityData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="name" />
            <YAxis />
            <ChartTooltip content={<ChartTooltipContent />} />
            <Bar dataKey="count" fill="var(--color-count)" radius={[4, 4, 0, 0]} />
          </BarChart>
        </ChartContainer>
      </div>
    )
  },
}

export const RealtimeMemoryChart: Story = {
  render: () => {
    const [data, setData] = useState<Array<{ time: string; memory: number }>>([])

    useEffect(() => {
      // Initial data
      const initialData = Array.from({ length: 30 }, (_, i) => ({
        time: formatTime(Date.now() - (29 - i) * 2000),
        memory: 40 + Math.random() * 20,
      }))
      setData(initialData)

      // Simulate real-time updates
      const interval = setInterval(() => {
        setData((prev) => {
          const newPoint = {
            time: formatTime(Date.now()),
            memory: 40 + Math.random() * 20,
          }
          return [...prev.slice(1), newPoint]
        })
      }, 2000)

      return () => clearInterval(interval)
    }, [])

    function formatTime(timestamp: number) {
      const date = new Date(timestamp)
      return `${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}:${date.getSeconds().toString().padStart(2, '0')}`
    }

    const chartConfig: ChartConfig = {
      memory: {
        label: 'Speicher (MB)',
        color: 'hsl(142, 76%, 36%)',
      },
    }

    return (
      <div className="w-full max-w-2xl">
        <h3 className="text-lg font-semibold mb-2">Echtzeit-Speicherverbrauch</h3>
        <p className="text-sm text-muted-foreground mb-4">
          Simulierte Echtzeit-Daten (aktualisiert alle 2 Sekunden)
        </p>
        <ChartContainer config={chartConfig} className="h-[300px] w-full">
          <AreaChart data={data}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="time" tick={{ fontSize: 10 }} interval="preserveStartEnd" />
            <YAxis domain={[30, 70]} />
            <ChartTooltip content={<ChartTooltipContent />} />
            <Area
              type="monotone"
              dataKey="memory"
              stroke="var(--color-memory)"
              fill="var(--color-memory)"
              fillOpacity={0.3}
              isAnimationActive={false}
            />
          </AreaChart>
        </ChartContainer>
      </div>
    )
  },
}
