import type { BarcodeDetector as BarcodeDetectorType } from 'barcode-detector/pure'
import KV from './KV'
import { boolBadge } from './badgeHelpers'
import {
  Badge,
  Button,
  CameraViewport,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@green-ecolution/ui'
import { useEffect, useRef, useState } from 'react'

interface EnvInfo {
  hasGetUserMedia: boolean
  hasNativeDetector: boolean
  supportedFormats: string[] | null
  hasVideoFrameCallback: boolean
  isSecureContext: boolean
  userAgent: string
}

interface ScanEntry {
  id: number
  timestamp: number
  rawValue: string
  format: string
  bboxWidth: number
  bboxHeight: number
}

type PermissionLabel = PermissionState | 'unknown' | 'unavailable'

const permissionBadge = (state: PermissionLabel) => {
  switch (state) {
    case 'granted':
      return <Badge variant="success">{state}</Badge>
    case 'denied':
      return <Badge variant="error">{state}</Badge>
    case 'prompt':
      return <Badge variant="warning">{state}</Badge>
    default:
      return <Badge variant="muted">{state}</Badge>
  }
}

const formatTime = (ts: number) => {
  const d = new Date(ts)
  return (
    d.toLocaleTimeString('de-DE', { hour12: false }) +
    '.' +
    String(d.getMilliseconds()).padStart(3, '0')
  )
}

const QRScannerDebugView = () => {
  const videoRef = useRef<HTMLVideoElement | null>(null)
  const streamRef = useRef<MediaStream | null>(null)
  const detectorRef = useRef<InstanceType<typeof BarcodeDetectorType> | null>(null)
  const runningRef = useRef(false)
  const lastScanRef = useRef<{ value: string; time: number } | null>(null)
  const scanIdRef = useRef(0)

  const [running, setRunning] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [flash, setFlash] = useState(false)
  const [scans, setScans] = useState<ScanEntry[]>([])
  const [env, setEnv] = useState<EnvInfo>({
    hasGetUserMedia: false,
    hasNativeDetector: false,
    supportedFormats: null,
    hasVideoFrameCallback: false,
    isSecureContext: false,
    userAgent: '',
  })
  const [permission, setPermission] = useState<PermissionLabel>('unknown')

  // Collect environment info once on mount
  useEffect(() => {
    if (typeof window === 'undefined') return

    const info: EnvInfo = {
      hasGetUserMedia: !!navigator.mediaDevices?.getUserMedia,
      hasNativeDetector: !!window.BarcodeDetector,
      supportedFormats: null,
      hasVideoFrameCallback: 'requestVideoFrameCallback' in HTMLVideoElement.prototype,
      isSecureContext: window.isSecureContext,
      userAgent: navigator.userAgent,
    }
    setEnv(info)

    const loadFormats = async () => {
      try {
        const Detector =
          window.BarcodeDetector ?? (await import('barcode-detector/pure')).BarcodeDetector
        const formats = await Detector.getSupportedFormats()
        setEnv((prev) => ({ ...prev, supportedFormats: [...formats] }))
      } catch (err) {
        console.error('Failed to query supported formats', err)
      }
    }
    void loadFormats()
  }, [])

  // Observe camera permission state
  useEffect(() => {
    let status: PermissionStatus | null = null
    const sub = async () => {
      try {
        status = await navigator.permissions.query({ name: 'camera' as PermissionName })
        setPermission(status.state)
        status.onchange = () => {
          if (status) setPermission(status.state)
        }
      } catch {
        setPermission('unavailable')
      }
    }
    void sub()
    return () => {
      if (status) status.onchange = null
    }
  }, [])

  const releaseStream = () => {
    runningRef.current = false
    streamRef.current?.getTracks().forEach((t) => t.stop())
    streamRef.current = null
    if (videoRef.current) videoRef.current.srcObject = null
  }

  const scheduleNext = (cb: () => void) => {
    if (env.hasVideoFrameCallback && videoRef.current) {
      videoRef.current.requestVideoFrameCallback(cb)
    } else {
      requestAnimationFrame(cb)
    }
  }

  const tick = async (): Promise<void> => {
    if (!runningRef.current) return
    const video = videoRef.current
    const detector = detectorRef.current
    if (video && detector && video.readyState >= 2 && !video.paused) {
      try {
        const codes = await detector.detect(video)
        if (!runningRef.current) return
        if (codes.length > 0) {
          const code = codes[0]
          const raw = code.rawValue ?? ''
          const now = Date.now()
          const last = lastScanRef.current
          // Dedupe: same value within 1s is considered the same sighting
          if (last?.value !== raw || now - last.time > 1000) {
            lastScanRef.current = { value: raw, time: now }
            const bbox = code.boundingBox
            const entry: ScanEntry = {
              id: ++scanIdRef.current,
              timestamp: now,
              rawValue: raw,
              format: String(code.format),
              bboxWidth: Math.round(bbox.width),
              bboxHeight: Math.round(bbox.height),
            }
            setScans((prev) => [entry, ...prev].slice(0, 100))
            setFlash(true)
            window.setTimeout(() => setFlash(false), 500)
          }
        }
      } catch (err) {
        console.debug('detect() threw', err)
      }
    }
    scheduleNext(() => void tick())
  }

  const handleStart = async () => {
    setError(null)
    if (!videoRef.current) return
    if (!navigator.mediaDevices?.getUserMedia) {
      setError('getUserMedia not available (insecure context?)')
      return
    }
    try {
      if (!detectorRef.current) {
        const Detector =
          window.BarcodeDetector ?? (await import('barcode-detector/pure')).BarcodeDetector
        detectorRef.current = new Detector({ formats: ['qr_code'] })
      }
      const stream = await navigator.mediaDevices.getUserMedia({
        video: { facingMode: { ideal: 'environment' } },
        audio: false,
      })
      streamRef.current = stream
      videoRef.current.srcObject = stream
      await videoRef.current.play()
      runningRef.current = true
      setRunning(true)
      void tick()
    } catch (err) {
      const message = err instanceof Error ? `${err.name}: ${err.message}` : String(err)
      setError(message)
      releaseStream()
      setRunning(false)
    }
  }

  const handleStop = () => {
    releaseStream()
    setRunning(false)
  }

  const handleClear = () => {
    setScans([])
    lastScanRef.current = null
  }

  // Release camera on unmount
  useEffect(() => {
    return () => {
      releaseStream()
    }
  }, [])

  return (
    <div className="flex flex-col gap-6">
      {/* Environment info */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card variant="outlined">
          <CardHeader>
            <CardTitle className="text-base">Umgebung</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-2 text-sm">
            <KV label="Secure Context">{boolBadge(env.isSecureContext)}</KV>
            <KV label="getUserMedia">{boolBadge(env.hasGetUserMedia)}</KV>
            <KV label="Native BarcodeDetector">{boolBadge(env.hasNativeDetector)}</KV>
            <KV label="requestVideoFrameCallback">{boolBadge(env.hasVideoFrameCallback)}</KV>
            <KV label="Unterstützte Formate">
              <span className="font-mono text-xs break-all">
                {env.supportedFormats
                  ? env.supportedFormats.length > 0
                    ? env.supportedFormats.join(', ')
                    : '—'
                  : 'lädt …'}
              </span>
            </KV>
            <KV label="User-Agent">
              <span className="font-mono text-xs break-all text-muted-foreground">
                {env.userAgent}
              </span>
            </KV>
          </CardContent>
        </Card>

        <Card variant="outlined">
          <CardHeader>
            <CardTitle className="text-base">Kamera-Permission</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-2 text-sm">
            <KV label="navigator.permissions">{permissionBadge(permission)}</KV>
            <KV label="Scanner läuft">{boolBadge(running)}</KV>
            <KV label="Erkannte Scans (Session)">
              <span className="font-mono">{scans.length}</span>
            </KV>
            {error && (
              <p className="mt-2 text-xs font-mono text-red break-all" role="alert">
                {error}
              </p>
            )}
          </CardContent>
        </Card>
      </div>

      {/* Controls */}
      <div className="flex flex-wrap gap-2">
        <Button size="sm" onClick={handleStart} disabled={running}>
          Scanner starten
        </Button>
        <Button size="sm" variant="outline" onClick={handleStop} disabled={!running}>
          Scanner stoppen
        </Button>
        <Button size="sm" variant="ghost" onClick={handleClear} disabled={scans.length === 0}>
          Log leeren
        </Button>
      </div>

      {/* Viewport */}
      <div className="max-w-md">
        <CameraViewport
          videoRef={videoRef}
          state={running ? 'scanning' : 'inactive'}
          flash={flash}
          ariaLabel="Debug Kamera-Vorschau"
        />
      </div>

      {/* Scan log */}
      <Card variant="outlined">
        <CardHeader>
          <CardTitle className="text-base">
            Scan-Log <span className="text-muted-foreground font-normal">({scans.length})</span>
          </CardTitle>
        </CardHeader>
        <CardContent className="pt-0 px-0">
          {scans.length === 0 ? (
            <p className="px-6 py-4 text-sm text-muted-foreground">
              Noch keine QR-Codes erkannt. Starte den Scanner und halte einen QR-Code in den
              Viewport.
            </p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-32">Zeit</TableHead>
                  <TableHead className="w-24">Format</TableHead>
                  <TableHead className="w-28">BBox (px)</TableHead>
                  <TableHead>Raw Value</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {scans.map((s) => (
                  <TableRow key={s.id}>
                    <TableCell className="font-mono text-xs">{formatTime(s.timestamp)}</TableCell>
                    <TableCell className="font-mono text-xs">{s.format}</TableCell>
                    <TableCell className="font-mono text-xs">
                      {s.bboxWidth}×{s.bboxHeight}
                    </TableCell>
                    <TableCell className="font-mono text-xs break-all">{s.rawValue}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
    </div>
  )
}

export default QRScannerDebugView
