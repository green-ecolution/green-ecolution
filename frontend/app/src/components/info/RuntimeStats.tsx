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
import { Activity, Cpu, Database, MemoryStick, RefreshCw } from 'lucide-react'
import type { RuntimeStatsResponse } from '@green-ecolution/backend-client'
import { basePath } from '@/api/backendApi'
import useStore from '@/store/store'

interface ChartDataPoint {
  time: string
  memory: number
}

const chartConfig: ChartConfig = {
  memory: {
    label: 'RSS (MB)',
    color: 'hsl(142, 76%, 36%)',
  },
}

const MAX_RECONNECT_ATTEMPTS = 5

function formatBytes(bytes: number): string {
  const mb = bytes / 1024 / 1024
  return `${mb.toFixed(1)} MB`
}

function formatTime(ms: number): string {
  const d = new Date(ms)
  return `${d.getHours().toString().padStart(2, '0')}:${d.getMinutes().toString().padStart(2, '0')}:${d.getSeconds().toString().padStart(2, '0')}`
}

export function RuntimeStats() {
  const [stats, setStats] = useState<RuntimeStatsResponse | null>(null)
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
    const delay = Math.min(1000 * 2 ** reconnectAttemptsRef.current, 30000)
    reconnectAttemptsRef.current++
    setConnectionStatus('connecting')
    setError(`Verbindung unterbrochen, reconnect in ${delay / 1000}s...`)
    reconnectTimeoutRef.current = setTimeout(() => {
      if (mountedRef.current && connectRef.current) connectRef.current()
    }, delay)
  }, [])

  const connect = useCallback(() => {
    if (!mountedRef.current) return
    if (wsRef.current) {
      wsRef.current.close()
      wsRef.current = null
    }
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current)
      reconnectTimeoutRef.current = null
    }
    setConnectionStatus('connecting')
    setError(null)

    const accessToken = useStore.getState().token?.accessToken
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const host = window.location.host
    const tokenParam = accessToken ? `?token=${encodeURIComponent(accessToken)}` : ''
    const wsUrl = `${protocol}//${host}${basePath}/v1/ws/stats${tokenParam}`

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
        const data = JSON.parse(event.data) as RuntimeStatsResponse
        if (typeof data.memory?.residentBytes !== 'number') {
          console.error('Invalid WebSocket data format')
          return
        }
        setStats(data)
        setChartData((prev) => {
          const newPoint: ChartDataPoint = {
            time: formatTime(data.timestamp),
            memory: data.memory.residentBytes / 1024 / 1024,
          }
          return [...prev, newPoint].slice(-60)
        })
      } catch (e) {
        console.error('Failed to parse WebSocket message:', e)
      }
    }

    ws.onclose = (event) => {
      if (!mountedRef.current) return
      wsRef.current = null
      if (event.code === 1000 || event.code === 1001) {
        setConnectionStatus('disconnected')
        return
      }
      scheduleReconnect()
    }

    ws.onerror = () => {
      if (!mountedRef.current) return
      setError('WebSocket-Fehler aufgetreten')
    }
  }, [scheduleReconnect])

  useEffect(() => {
    connectRef.current = connect
  }, [connect])

  useEffect(() => {
    mountedRef.current = true
    const initTimeout = setTimeout(() => {
      if (mountedRef.current) connect()
    }, 100)
    return () => {
      mountedRef.current = false
      clearTimeout(initTimeout)
      if (reconnectTimeoutRef.current) clearTimeout(reconnectTimeoutRef.current)
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

  if (!stats) return null

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
