import { useCallback, useEffect, useState } from 'react'
import type { RuntimeStatsResponse } from '@green-ecolution/backend-client'
import { basePath } from '@/api/backendApi'
import useStore from '@/store/store'

export type RuntimeStatsConnectionStatus = 'connecting' | 'connected' | 'disconnected'

export interface RuntimeStatsHistoryPoint {
  timestamp: number
  memoryBytes: number
}

export interface UseRuntimeStatsSocketResult {
  status: RuntimeStatsConnectionStatus
  stats: RuntimeStatsResponse | null
  history: RuntimeStatsHistoryPoint[]
  error: string | null
  reconnect: () => void
}

export interface UseRuntimeStatsSocketOptions {
  historySize?: number
}

const MAX_RECONNECT_ATTEMPTS = 5
const MAX_BACKOFF_MS = 30_000
const DEFAULT_HISTORY_SIZE = 60

function buildWsUrl(token: string | undefined): string {
  const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
  const host = window.location.host
  const tokenParam = token ? `?token=${encodeURIComponent(token)}` : ''
  return `${protocol}//${host}${basePath}/api/v1/ws/stats${tokenParam}`
}

function isRuntimeStatsResponse(value: unknown): value is RuntimeStatsResponse {
  if (typeof value !== 'object' || value === null) return false
  const memory = (value as { memory?: { residentBytes?: unknown } }).memory
  return typeof memory?.residentBytes === 'number'
}

export function useRuntimeStatsSocket(
  options: UseRuntimeStatsSocketOptions = {},
): UseRuntimeStatsSocketResult {
  const historySize = options.historySize ?? DEFAULT_HISTORY_SIZE

  const [status, setStatus] = useState<RuntimeStatsConnectionStatus>('connecting')
  const [stats, setStats] = useState<RuntimeStatsResponse | null>(null)
  const [history, setHistory] = useState<RuntimeStatsHistoryPoint[]>([])
  const [error, setError] = useState<string | null>(null)
  const [attempt, setAttempt] = useState(0)

  useEffect(() => {
    let ws: WebSocket | null = null
    let reconnectTimer: ReturnType<typeof setTimeout> | null = null
    let reconnects = 0
    let cancelled = false

    const connect = () => {
      if (cancelled) return
      setStatus('connecting')
      setError(null)

      const token = useStore.getState().token?.accessToken
      const socket = new WebSocket(buildWsUrl(token))
      ws = socket

      socket.onopen = () => {
        if (cancelled) return
        reconnects = 0
        setStatus('connected')
      }

      socket.onmessage = (event: MessageEvent<string>) => {
        if (cancelled) return
        try {
          const data: unknown = JSON.parse(event.data)
          if (!isRuntimeStatsResponse(data)) {
            console.error('Invalid WebSocket data format')
            return
          }
          setStats(data)
          setHistory((prev) =>
            [
              ...prev,
              { timestamp: data.timestamp, memoryBytes: data.memory.residentBytes },
            ].slice(-historySize),
          )
        } catch (e) {
          console.error('Failed to parse WebSocket message:', e)
        }
      }

      socket.onclose = (event) => {
        if (cancelled) return
        ws = null

        if (event.code === 1000 || event.code === 1001) {
          setStatus('disconnected')
          return
        }
        if (reconnects >= MAX_RECONNECT_ATTEMPTS) {
          setStatus('disconnected')
          setError('Maximale Verbindungsversuche erreicht')
          return
        }

        const delay = Math.min(1000 * 2 ** reconnects, MAX_BACKOFF_MS)
        reconnects++
        setStatus('connecting')
        setError(`Verbindung unterbrochen, reconnect in ${delay / 1000}s…`)
        reconnectTimer = setTimeout(connect, delay)
      }

      socket.onerror = () => {
        if (cancelled) return
        setError('WebSocket-Fehler aufgetreten')
      }
    }

    connect()

    return () => {
      cancelled = true
      if (reconnectTimer) clearTimeout(reconnectTimer)
      if (ws) {
        ws.close(1000, 'Component unmounted')
        ws = null
      }
    }
  }, [attempt, historySize])

  const reconnect = useCallback(() => {
    setAttempt((a) => a + 1)
  }, [])

  return { status, stats, history, error, reconnect }
}
