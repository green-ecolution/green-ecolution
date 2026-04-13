import type { GeolocationFix, GeolocationStatus } from '@/hooks/useGeolocation'
import { AccuracyBadge } from '@green-ecolution/ui'
import { Loader2, MapPinOff } from 'lucide-react'

interface InlineGPSReadoutProps {
  position: GeolocationFix | null
  status: GeolocationStatus
}

const InlineGPSReadout = ({ position, status }: InlineGPSReadoutProps) => {
  const isLoading = status === 'requesting' || status === 'idle'
  const isUnavailable = status === 'denied' || status === 'unsupported' || status === 'error'

  return (
    <div className="flex flex-col gap-2 border-t border-dark-100 pt-3">
      <div className="flex items-center justify-between gap-2">
        <span className="text-xs uppercase tracking-widest text-muted-foreground">
          Erfasster Standort
        </span>
        <AccuracyBadge accuracyMeters={position?.accuracy ?? null} />
      </div>
      {position ? (
        <code className="font-mono text-sm md:text-base break-all bg-dark-50 rounded-lg px-3 py-2 border border-dark-100">
          {position.latitude.toFixed(6)}, {position.longitude.toFixed(6)}
        </code>
      ) : isLoading ? (
        <p className="flex items-center gap-2 text-sm text-muted-foreground px-3 py-2">
          <Loader2 className="size-4 animate-spin" aria-hidden />
          GPS-Standort wird ermittelt …
        </p>
      ) : isUnavailable ? (
        <p className="flex items-center gap-2 text-sm text-red px-3 py-2">
          <MapPinOff className="size-4" aria-hidden />
          Standort konnte nicht ermittelt werden.
        </p>
      ) : null}
    </div>
  )
}

export default InlineGPSReadout
