import { useMemo } from 'react'
import { Area, AreaChart, CartesianGrid, XAxis, YAxis } from 'recharts'
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
  StatusCard,
  type ChartConfig,
} from '@green-ecolution/ui'
import { Activity, Cpu, Database, MemoryStick, RefreshCw } from 'lucide-react'
import { useRuntimeStatsSocket } from '@/hooks/useRuntimeStatsSocket'

interface ChartDataPoint {
  time: string
  memory: number
}

const BYTES_PER_MB = 1024 * 1024

const chartConfig: ChartConfig = {
  memory: {
    label: 'RSS (MB)',
    color: 'hsl(142, 76%, 36%)',
  },
}

function formatBytes(bytes: number): string {
  return `${(bytes / BYTES_PER_MB).toFixed(1)} MB`
}

function formatTime(ms: number): string {
  const d = new Date(ms)
  const hh = d.getHours().toString().padStart(2, '0')
  const mm = d.getMinutes().toString().padStart(2, '0')
  const ss = d.getSeconds().toString().padStart(2, '0')
  return `${hh}:${mm}:${ss}`
}

export function RuntimeStats() {
  const { status, stats, history, error, reconnect } = useRuntimeStatsSocket()

  const chartData = useMemo<ChartDataPoint[]>(
    () =>
      history.map((point) => ({
        time: formatTime(point.timestamp),
        memory: point.memoryBytes / BYTES_PER_MB,
      })),
    [history],
  )

  if (status === 'connecting' && !stats) {
    return (
      <div className="flex items-center gap-2 text-dark-600">
        <RefreshCw className="size-4 animate-spin" />
        Verbinde mit Runtime Stats...
      </div>
    )
  }

  if (status === 'disconnected') {
    return (
      <div className="text-dark-600">
        <p>
          WebSocket-Verbindung getrennt.{' '}
          <button onClick={reconnect} className="text-green-dark hover:underline">
            Erneut verbinden
          </button>
        </p>
        {error && <p className="text-sm text-red mt-2">{error}</p>}
      </div>
    )
  }

  if (!stats) {
    return (
      <div className="flex items-center gap-2 text-dark-600">
        <RefreshCw className="size-4 animate-spin" />
        Warte auf Daten...
      </div>
    )
  }

  return (
    <div className="space-y-8">
      <div className="grid gap-5 md:grid-cols-2 lg:grid-cols-4">
        <StatusCard
          status="green-dark"
          indicator="dot"
          icon={<MemoryStick className="size-4" />}
          label="Speicher (RSS)"
          value={formatBytes(stats.memory.residentBytes)}
          description={`Virtual: ${formatBytes(stats.memory.virtualBytes)}`}
        />
        <StatusCard
          icon={<Cpu className="size-4" />}
          label="CPU"
          value={`${stats.cpu.usagePercent.toFixed(1)}%`}
          description={`${stats.cpu.cores} Cores`}
        />
        <StatusCard
          icon={<Database className="size-4" />}
          label="DB Pool"
          value={`${stats.dbPool.active} / ${stats.dbPool.max}`}
          description={`${stats.dbPool.idle} idle`}
        />
        <StatusCard
          icon={<Activity className="size-4" />}
          label="Tokio"
          value={`${stats.tokio.workerThreads} Workers`}
          description={`${stats.tokio.blockingThreads} Blocking`}
        />
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Activity className="size-5" />
            Speicherverbrauch (Live)
          </CardTitle>
        </CardHeader>
        <CardContent>
          {chartData.length > 1 ? (
            <ChartContainer config={chartConfig} className="h-[300px] w-full">
              <AreaChart data={chartData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="time" tick={{ fontSize: 10 }} interval="preserveStartEnd" />
                <YAxis
                  tickFormatter={(v: number) => `${v.toFixed(0)}`}
                  label={{ value: 'MB', angle: -90, position: 'insideLeft' }}
                />
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
          ) : (
            <div className="h-[300px] flex items-center justify-center text-dark-600">
              Sammle Daten...
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  )
}
