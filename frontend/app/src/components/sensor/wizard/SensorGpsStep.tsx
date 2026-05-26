import GeolocationPermissionNotice from '@/components/geolocation/GeolocationPermissionNotice'
import GPSStatusCard from '@/components/geolocation/GPSStatusCard'
import LocationMapPreview from '@/components/geolocation/LocationMapPreview'
import type { GeolocationFix, GeolocationStatus } from '@/hooks/useGeolocation'
import { Button } from '@green-ecolution/ui'
import { Crosshair, Loader2, MapPin } from 'lucide-react'

interface SensorGpsStepProps {
  position: GeolocationFix | null
  status: GeolocationStatus
  errorMessage: string | null
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

const SensorGpsStep = ({
  position,
  status,
  errorMessage,
  onRelocate,
}: SensorGpsStepProps) => {
  const noticeStatus: 'denied' | 'unsupported' | 'error' | null =
    status === 'denied' || status === 'unsupported' || status === 'error' ? status : null

  return (
    <div className="space-y-6">
      <header className="space-y-2">
        <h1 className="font-lato font-bold text-3xl lg:text-4xl">Standort bestätigen</h1>
        <p className="text-sm text-muted-foreground max-w-prose">
          Überprüfe den erfassten GPS-Standort. Bei Bedarf kannst du erneut lokalisieren.
        </p>
      </header>
      <div className="grid gap-4 md:grid-cols-2 md:gap-6">
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
        <div className="md:col-span-1">
          <GPSStatusCard fix={position} title="Erfasster Standort" />
        </div>
      </div>
      <Button
        variant="outline"
        onClick={onRelocate}
        disabled={status === 'requesting'}
        className="w-full sm:w-auto"
      >
        <MapPin className="size-4" />
        Erneut lokalisieren
      </Button>
    </div>
  )
}

export default SensorGpsStep
