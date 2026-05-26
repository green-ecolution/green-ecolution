import type { SubmissionState } from './state'
import { AccuracyBadge, Button, CopyableText, InlineAlert } from '@green-ecolution/ui'
import { Loader2, ShieldCheck, TreeDeciduous } from 'lucide-react'

interface SensorReviewStepProps {
  sensorId: string
  treeNumber: string
  treeSpecies: string
  position: {
    latitude: number
    longitude: number
    accuracy: number
    timestamp: number
  }
  status: SubmissionState
  errorMessage: string | null
  onActivate: () => void
}

const formatCoordinate = (n: number) => n.toFixed(5)

const SensorReviewStep = ({
  sensorId,
  treeNumber,
  treeSpecies,
  position,
  status,
  errorMessage,
  onActivate,
}: SensorReviewStepProps) => {
  const isPending = status === 'pending'
  const isError = status === 'error'

  return (
    <div className="space-y-6">
      <header className="space-y-2">
        <h1 className="font-lato font-bold text-3xl lg:text-4xl">Zuordnung prüfen</h1>
        <p className="text-sm text-muted-foreground max-w-prose">
          Bestätige Sensor, Baum und Standort. Mit „Sensor aktivieren" wird die Verknüpfung
          gespeichert.
        </p>
      </header>

      <div className="grid gap-4 md:grid-cols-2">
        <CopyableText value={sensorId} label="Sensor-ID" className="md:col-span-2" />

        <div className="rounded-xl border border-dark-100 p-4">
          <p className="text-xs uppercase tracking-wide font-semibold text-muted-foreground mb-2 flex items-center gap-1.5">
            <TreeDeciduous className="size-3.5" aria-hidden />
            Zugeordneter Baum
          </p>
          <div className="space-y-1">
            <p className="font-semibold">{treeSpecies}</p>
            <p className="font-mono text-xs text-dark-600">{treeNumber}</p>
          </div>
        </div>

        <div className="rounded-xl border border-dark-100 p-4">
          <p className="text-xs uppercase tracking-wide font-semibold text-muted-foreground mb-2">
            Standort
          </p>
          <p className="text-sm tabular-nums">
            {formatCoordinate(position.latitude)}, {formatCoordinate(position.longitude)}
          </p>
          <div className="mt-2">
            <AccuracyBadge accuracyMeters={position.accuracy} />
          </div>
        </div>
      </div>

      {isError && errorMessage && <InlineAlert variant="destructive" description={errorMessage} />}

      <Button onClick={onActivate} disabled={isPending} className="w-full sm:w-auto">
        {isPending ? (
          <>
            <Loader2 className="size-4 animate-spin" />
            Aktiviere …
          </>
        ) : (
          <>
            <ShieldCheck className="size-4" />
            Sensor aktivieren
          </>
        )}
      </Button>
    </div>
  )
}

export default SensorReviewStep
