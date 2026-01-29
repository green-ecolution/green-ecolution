import { useCallback, useEffect, useRef, useState } from 'react'
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
import { Activity, Cpu, HardDrive, MemoryStick, RefreshCw } from 'lucide-react'
import { basePath } from '@/api/backendApi'

interface RuntimeStats {
  alloc: number
  totalAlloc: number
  sys: number
  heapAlloc: number
  heapSys: number
  heapInuse: number
  numGoroutine: number
  numGC: number
  pauseTotalNs: number
  numCPU: number
  timestamp: number
}

interface ChartDataPoint {
  time: string
  memory: number
  heap: number
}

const chartConfig: ChartConfig = {
  memory: {
    label: 'Alloc (MB)',
    color: 'hsl(142, 76%, 36%)',
  },
  heap: {
    label: 'Heap (MB)',
    color: 'hsl(217, 91%, 60%)',
  },
}

function formatBytes(bytes: number): string {
  const mb = bytes / 1024 / 1024
  return `${mb.toFixed(1)} MB`
}

const MAX_RECONNECT_ATTEMPTS = 5

function formatTime(timestamp: number): string {
  const date = new Date(timestamp)
  return `${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}:${date.getSeconds().toString().padStart(2, '0')}`
}

export function RuntimeStats() {
  const [stats, setStats] = useState<RuntimeStats | null>(null)
  const [chartData, setChartData] = useState<ChartDataPoint[]>([])
  const [connectionStatus, setConnectionStatus] = useState<
    'connecting' | 'connected' | 'disconnected'
  >('connecting')
  const [error, setError] = useState<string | null>(null)
  const wsRef = useRef<WebSocket | null>(null)
  const mountedRef = useRef(true)
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)
  const reconnectAttemptsRef = useRef(0)
  const connectRef = useRef<(() => void) | null>(null)

  const scheduleReconnect = useCallback(() => {
    if (reconnectAttemptsRef.current >= MAX_RECONNECT_ATTEMPTS) {
      setConnectionStatus('disconnected')
      setError('Maximale Verbindungsversuche erreicht')
      return
    }

    // Auto-reconnect with exponential backoff (max 30s)
    const delay = Math.min(1000 * Math.pow(2, reconnectAttemptsRef.current), 30000)
    reconnectAttemptsRef.current++

    setConnectionStatus('connecting')
    setError(`Verbindung unterbrochen, reconnect in ${delay / 1000}s...`)

    reconnectTimeoutRef.current = setTimeout(() => {
      if (mountedRef.current && connectRef.current) {
        connectRef.current()
      }
    }, delay)
  }, [])

  const connect = useCallback(() => {
    // Don't connect if unmounted
    if (!mountedRef.current) return

    // Close existing connection if any
    if (wsRef.current) {
      wsRef.current.close()
      wsRef.current = null
    }

    // Clear any pending reconnect
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current)
      reconnectTimeoutRef.current = null
    }

    setConnectionStatus('connecting')
    setError(null)

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const host = window.location.host
    const wsUrl = `${protocol}//${host}${basePath}/v1/ws/stats`

    const ws = new WebSocket(wsUrl)
    wsRef.current = ws

    ws.onopen = () => {
      if (!mountedRef.current) return
      reconnectAttemptsRef.current = 0
      setConnectionStatus('connected')
    }

    ws.onmessage = (event: MessageEvent<string>) => {
      if (!mountedRef.current) return
      try {
        const data = JSON.parse(event.data) as RuntimeStats
        if (typeof data.alloc !== 'number') {
          console.error('Invalid WebSocket data format')
          return
        }
        setStats(data)

        setChartData((prev) => {
          const newPoint: ChartDataPoint = {
            time: formatTime(data.timestamp),
            memory: data.alloc / 1024 / 1024,
            heap: data.heapAlloc / 1024 / 1024,
          }
          const updated = [...prev, newPoint]
          // Keep last 60 data points (~2 minutes at 2s interval)
          return updated.slice(-60)
        })
      } catch (error) {
        console.error('Failed to parse WebSocket message:', error)
      }
    }

    ws.onclose = (event) => {
      if (!mountedRef.current) return
      wsRef.current = null

      // Code 1000 = normal closure, 1001 = going away (tab close)
      if (event.code === 1000 || event.code === 1001) {
        setConnectionStatus('disconnected')
        return
      }

      scheduleReconnect()
    }

    ws.onerror = () => {
      // onerror is always followed by onclose, so we handle reconnect there
      if (!mountedRef.current) return
      setError('WebSocket-Fehler aufgetreten')
    }
  }, [scheduleReconnect])

  // Keep connectRef in sync
  useEffect(() => {
    connectRef.current = connect
  }, [connect])

  useEffect(() => {
    mountedRef.current = true

    // Small delay to ensure component is fully mounted (helps with Strict Mode)
    const initTimeout = setTimeout(() => {
      if (mountedRef.current) {
        connect()
      }
    }, 100)

    return () => {
      mountedRef.current = false
      clearTimeout(initTimeout)

      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current)
      }

      if (wsRef.current) {
        wsRef.current.close(1000, 'Component unmounted')
        wsRef.current = null
      }
    }
  }, [connect])

  if (connectionStatus === 'connecting') {
    return (
      <div className="flex items-center gap-2 text-dark-600">
        <RefreshCw className="size-4 animate-spin" />
        Verbinde mit Runtime Stats...
      </div>
    )
  }

  const handleManualReconnect = () => {
    reconnectAttemptsRef.current = 0
    connect()
  }

  if (connectionStatus === 'disconnected') {
    return (
      <div className="text-dark-600">
        <p>
          WebSocket-Verbindung getrennt.{' '}
          <button onClick={handleManualReconnect} className="text-green-dark hover:underline">
            Erneut verbinden
          </button>
        </p>
        {error && <p className="text-sm text-red mt-2">{error}</p>}
      </div>
    )
  }

  if (connectionStatus === 'connected' && !stats) {
    return (
      <div className="flex items-center gap-2 text-dark-600">
        <RefreshCw className="size-4 animate-spin" />
        Warte auf Daten...
      </div>
    )
  }

  if (!stats) {
    return null
  }

  return (
    <div className="space-y-8">
      <div className="grid gap-5 md:grid-cols-2 lg:grid-cols-4">
        <StatusCard
          status="green-dark"
          indicator="dot"
          icon={<MemoryStick className="size-4" />}
          label="Speicher (Alloc)"
          value={formatBytes(stats.alloc)}
          description="Aktuell allokierter Speicher"
        />
        <StatusCard
          icon={<HardDrive className="size-4" />}
          label="Heap"
          value={formatBytes(stats.heapAlloc)}
          description={`Heap Inuse: ${formatBytes(stats.heapInuse)}`}
        />
        <StatusCard
          icon={<Activity className="size-4" />}
          label="Goroutines"
          value={stats.numGoroutine.toString()}
          description={`GC Cycles: ${stats.numGC}`}
        />
        <StatusCard
          icon={<Cpu className="size-4" />}
          label="CPU Cores"
          value={stats.numCPU.toString()}
          description={`System: ${formatBytes(stats.sys)}`}
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
                  tickFormatter={(value: number) => `${value.toFixed(0)}`}
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
                <Area
                  type="monotone"
                  dataKey="heap"
                  stroke="var(--color-heap)"
                  fill="var(--color-heap)"
                  fillOpacity={0.2}
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
