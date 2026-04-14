import GPSStatusCard from '@/components/geolocation/GPSStatusCard'
import LocationMapPreview from '@/components/geolocation/LocationMapPreview'
import useGeolocation, { type GeolocationStatus } from '@/hooks/useGeolocation'
import KV from './KV'
import { boolBadge } from './badgeHelpers'
import {
  Badge,
  Button,
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
import { Compass, Crosshair, Loader2, MapPin } from 'lucide-react'
import { useEffect, useState } from 'react'

interface EnvInfo {
  hasGeolocation: boolean
  hasPermissionsApi: boolean
  isSecureContext: boolean
  userAgent: string
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

const statusBadge = (status: GeolocationStatus) => {
  switch (status) {
    case 'watching':
      return <Badge variant="success">watching</Badge>
    case 'requesting':
      return <Badge variant="warning">requesting</Badge>
    case 'denied':
      return <Badge variant="error">denied</Badge>
    case 'unsupported':
      return <Badge variant="error">unsupported</Badge>
    case 'error':
      return <Badge variant="error">error</Badge>
    default:
      return <Badge variant="muted">idle</Badge>
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

const EmptyMapState = ({ status }: { status: GeolocationStatus }) => {
  const isLoading = status === 'requesting'
  return (
    <div
      className="flex aspect-[4/3] sm:aspect-[16/10] lg:aspect-auto lg:h-full lg:min-h-72 w-full flex-col items-center justify-center gap-3 rounded-2xl border border-dashed border-dark-200 bg-dark-50 text-muted-foreground"
      role="status"
      aria-live="polite"
    >
      {isLoading ? (
        <>
          <Loader2 className="size-6 animate-spin" aria-hidden />
          <p className="text-sm">GPS wird ermittelt …</p>
        </>
      ) : (
        <>
          <Crosshair className="size-6" aria-hidden />
          <p className="text-sm">Noch keine Position erfasst</p>
        </>
      )}
    </div>
  )
}

const GeolocationDebugView = () => {
  const { status, position, history, errorMessage, start, stop, reset } = useGeolocation({
    trackHistory: true,
  })

  const [permission, setPermission] = useState<PermissionLabel>('unknown')
  // Captured once on first render — window/navigator are stable within a session.
  const [env] = useState<EnvInfo>(() => {
    if (typeof window === 'undefined') {
      return {
        hasGeolocation: false,
        hasPermissionsApi: false,
        isSecureContext: false,
        userAgent: '',
      }
    }
    return {
      hasGeolocation: !!navigator.geolocation,
      hasPermissionsApi: typeof navigator.permissions?.query === 'function',
      isSecureContext: window.isSecureContext,
      userAgent: navigator.userAgent,
    }
  })

  // Observe geolocation permission state.
  useEffect(() => {
    let permStatus: PermissionStatus | null = null
    let handler: (() => void) | null = null
    const sub = async () => {
      try {
        permStatus = await navigator.permissions.query({ name: 'geolocation' as PermissionName })
        setPermission(permStatus.state)
        handler = () => {
          if (permStatus) setPermission(permStatus.state)
        }
        permStatus.addEventListener('change', handler)
      } catch {
        setPermission('unavailable')
      }
    }
    void sub()
    return () => {
      if (permStatus && handler) {
        permStatus.removeEventListener('change', handler)
      }
    }
  }, [])

  const isWatching = status === 'watching' || status === 'requesting'

  return (
    <div className="flex flex-col gap-6">
      {/* 1. Hero readout band */}
      <div className="grid gap-4 lg:grid-cols-5">
        <div className="lg:col-span-3">
          {position ? (
            <LocationMapPreview
              latitude={position.latitude}
              longitude={position.longitude}
              accuracyMeters={position.accuracy}
              className="aspect-[4/3] sm:aspect-[16/10] lg:aspect-auto lg:h-full lg:min-h-72"
              ariaLabel="Live-Karte mit aktueller GPS-Position"
              interactive
            />
          ) : (
            <EmptyMapState status={status} />
          )}
        </div>
        <div className="lg:col-span-2">
          <GPSStatusCard fix={position} />
        </div>
      </div>

      {/* 2. Controls */}
      <div className="flex flex-wrap gap-2">
        <Button size="sm" onClick={() => void start()} disabled={isWatching}>
          <MapPin className="size-4" />
          Ortung starten
        </Button>
        <Button size="sm" variant="outline" onClick={stop} disabled={!isWatching}>
          Ortung stoppen
        </Button>
        <Button size="sm" variant="ghost" onClick={reset} disabled={history.length === 0}>
          Log leeren
        </Button>
      </div>

      {/* 3. Diagnostics grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <Card variant="outlined">
          <CardHeader>
            <CardTitle className="text-base">Umgebung</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-2 text-sm">
            <KV label="Secure Context">{boolBadge(env.isSecureContext)}</KV>
            <KV label="navigator.geolocation">{boolBadge(env.hasGeolocation)}</KV>
            <KV label="Permissions API">{boolBadge(env.hasPermissionsApi)}</KV>
            <KV label="User-Agent">
              <span className="font-mono text-xs break-all text-muted-foreground">
                {env.userAgent}
              </span>
            </KV>
          </CardContent>
        </Card>

        <Card variant="outlined">
          <CardHeader>
            <CardTitle className="text-base">Permission &amp; Sensoren</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-2 text-sm">
            <KV label="navigator.permissions">{permissionBadge(permission)}</KV>
            <KV label="Hook-Status">{statusBadge(status)}</KV>
            <KV label="Erfasste Fixes">
              <span className="font-mono">{history.length}</span>
            </KV>
            <KV label="Heading">
              <span className="font-mono">
                {position?.heading != null ? `${position.heading.toFixed(1)}°` : '—'}
              </span>
            </KV>
            <KV label="Speed">
              <span className="font-mono">
                {position?.speed != null ? `${position.speed.toFixed(2)} m/s` : '—'}
              </span>
            </KV>
            {errorMessage && (
              <p className="mt-2 text-xs font-mono text-red break-all" role="alert">
                <Compass className="inline size-3 mr-1" aria-hidden />
                {errorMessage}
              </p>
            )}
          </CardContent>
        </Card>
      </div>

      {/* 4. Verlauf */}
      <Card variant="outlined">
        <CardHeader>
          <CardTitle className="text-base">
            Verlauf <span className="text-muted-foreground font-normal">({history.length})</span>
          </CardTitle>
        </CardHeader>
        <CardContent className="pt-0 px-0">
          {history.length === 0 ? (
            <p className="px-6 py-4 text-sm text-muted-foreground">
              Noch keine Position erfasst. Starte die Ortung, um Live-Daten zu sehen.
            </p>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-32">Zeit</TableHead>
                  <TableHead>Lat / Lng</TableHead>
                  <TableHead className="w-28">Accuracy</TableHead>
                  <TableHead className="w-24">Heading</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {history.map((fix) => (
                  <TableRow key={fix.timestamp}>
                    <TableCell className="font-mono text-xs">{formatTime(fix.timestamp)}</TableCell>
                    <TableCell className="font-mono text-xs">
                      {fix.latitude.toFixed(6)}, {fix.longitude.toFixed(6)}
                    </TableCell>
                    <TableCell className="font-mono text-xs">
                      ± {fix.accuracy.toFixed(1)} m
                    </TableCell>
                    <TableCell className="font-mono text-xs">
                      {fix.heading != null ? `${fix.heading.toFixed(0)}°` : '—'}
                    </TableCell>
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

export default GeolocationDebugView
