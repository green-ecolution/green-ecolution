import { AccuracyBadge, Card, CardContent, CardHeader, CardTitle } from '@green-ecolution/ui'
import { MapPin } from 'lucide-react'
import type { GeolocationFix } from '@/hooks/useGeolocation'

interface GPSStatusCardProps {
  fix: GeolocationFix | null
  /** Optional headline override. */
  title?: string
}

const formatCoord = (value: number) => value.toFixed(6)

const formatTime = (timestamp: number) => {
  const d = new Date(timestamp)
  return d.toLocaleTimeString('de-DE', { hour12: false })
}

const formatAltitude = (alt: number | null, altAcc: number | null) => {
  if (alt == null) return '—'
  const accSuffix = altAcc != null ? ` ± ${altAcc.toFixed(0)} m` : ''
  return `${alt.toFixed(1)} m${accSuffix}`
}

const GPSStatusCard = ({ fix, title = 'Aktuelle Position' }: GPSStatusCardProps) => {
  return (
    <Card variant="outlined">
      <CardHeader className="flex flex-row items-center justify-between gap-3 space-y-0">
        <div className="flex items-center gap-2">
          <MapPin className="size-4 text-green-dark" aria-hidden />
          <CardTitle className="text-base">{title}</CardTitle>
        </div>
        <AccuracyBadge accuracyMeters={fix?.accuracy ?? null} />
      </CardHeader>
      <CardContent>
        <dl className="grid grid-cols-2 gap-x-6 gap-y-3 text-sm">
          <Field label="Breitengrad" value={fix ? formatCoord(fix.latitude) : '—'} mono />
          <Field label="Längengrad" value={fix ? formatCoord(fix.longitude) : '—'} mono />
          <Field label="Genauigkeit" value={fix ? `± ${fix.accuracy.toFixed(1)} m` : '—'} mono />
          <Field
            label="Höhe"
            value={formatAltitude(fix?.altitude ?? null, fix?.altitudeAccuracy ?? null)}
            mono
          />
          <Field
            label="Erfasst um"
            value={fix ? formatTime(fix.timestamp) : '—'}
            mono
            className="col-span-2"
          />
        </dl>
      </CardContent>
    </Card>
  )
}

interface FieldProps {
  label: string
  value: string
  mono?: boolean
  className?: string
}

const Field = ({ label, value, mono, className }: FieldProps) => (
  <div className={className}>
    <dt className="text-xs uppercase tracking-wide text-muted-foreground">{label}</dt>
    <dd className={mono ? 'mt-0.5 font-mono text-sm text-foreground' : 'mt-0.5 text-sm'}>
      {value}
    </dd>
  </div>
)

export default GPSStatusCard
