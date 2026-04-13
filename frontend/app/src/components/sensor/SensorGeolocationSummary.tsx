import GeolocationPermissionNotice from '@/components/geolocation/GeolocationPermissionNotice'
import GPSStatusCard from '@/components/geolocation/GPSStatusCard'
import LocationMapPreview from '@/components/geolocation/LocationMapPreview'
import type { GeolocationFix, GeolocationStatus } from '@/hooks/useGeolocation'
import {
  AccuracyBadge,
  Card,
  CardContent,
  Button,
  InlineAlert,
} from '@green-ecolution/ui'
import { CheckCircle2, Crosshair, Loader2, MapPin, RotateCw } from 'lucide-react'

interface SensorGeolocationSummaryProps {
  sensorId: string
  position: GeolocationFix | null
  status: GeolocationStatus
  errorMessage: string | null
  onScanAgain: () => void
  onRelocate: () => void
}

const MapPlaceholder = ({ status }: { status: GeolocationStatus }) => {
  const isLoading = status === 'requesting' || status === 'idle'
  return (
    <div
      role="status"
      aria-live="polite"
      className="flex aspect-[4/3] sm:aspect-[16/10] w-full flex-col items-center justify-center gap-3 rounded-2xl border border-dashed border-dark-200 bg-dark-50 text-muted-foreground"
    >
      {isLoading ? (
        <>
          <Loader2 className="size-6 animate-spin" aria-hidden />
          <p className="text-sm">GPS wird ermittelt …</p>
        </>
      ) : (
        <>
          <Crosshair className="size-6" aria-hidden />
          <p className="text-sm">Keine Position verfügbar</p>
        </>
      )}
    </div>
  )
}

const SensorGeolocationSummary = ({
  sensorId,
  position,
  status,
  errorMessage,
  onScanAgain,
  onRelocate,
}: SensorGeolocationSummaryProps) => {
  const noticeStatus: 'denied' | 'unsupported' | 'error' | null =
    status === 'denied' || status === 'unsupported' || status === 'error' ? status : null

  return (
    <div className="mx-auto w-full max-w-3xl pb-[env(safe-area-inset-bottom)]">
      {/* Status line */}
      <div className="flex items-center gap-2 text-sm font-medium text-green-dark mb-4">
        <CheckCircle2 className="size-4" aria-hidden />
        <span>Sensor erfasst</span>
      </div>

      <div className="grid gap-4 md:grid-cols-2 md:gap-6">
        {/* Sensor-ID — spans both columns */}
        <Card variant="outlined" className="md:col-span-2">
          <CardContent className="flex flex-col gap-3 p-5 sm:flex-row sm:items-end sm:justify-between">
            <div className="min-w-0">
              <p className="text-xs uppercase tracking-wide text-muted-foreground">Sensor-ID</p>
              <p className="mt-1 font-mono text-2xl font-semibold tracking-tight break-all">
                {sensorId}
              </p>
            </div>
            <AccuracyBadge accuracyMeters={position?.accuracy ?? null} />
          </CardContent>
        </Card>

        {/* Map (anchor) */}
        <div className="md:col-span-1">
          {position ? (
            <LocationMapPreview
              latitude={position.latitude}
              longitude={position.longitude}
              accuracyMeters={position.accuracy}
            />
          ) : noticeStatus ? (
            <GeolocationPermissionNotice
              status={noticeStatus}
              errorMessage={errorMessage}
              onRetry={noticeStatus === 'unsupported' ? undefined : onRelocate}
            />
          ) : (
            <MapPlaceholder status={status} />
          )}
        </div>

        {/* GPS status */}
        <div className="md:col-span-1">
          <GPSStatusCard fix={position} title="Erfasster Standort" />
        </div>

        {/* Actions */}
        <div className="grid grid-cols-1 gap-2 sm:grid-cols-2 md:col-span-2">
          <Button variant="outline" onClick={onScanAgain}>
            <RotateCw className="size-4" />
            Erneut scannen
          </Button>
          <Button onClick={onRelocate} disabled={status === 'requesting'}>
            <MapPin className="size-4" />
            Erneut lokalisieren
          </Button>
        </div>

        <InlineAlert
          variant="warning"
          description="Die Speicherung des Sensors ist noch nicht implementiert."
          className="md:col-span-2 w-full"
        />
      </div>
    </div>
  )
}

export default SensorGeolocationSummary
