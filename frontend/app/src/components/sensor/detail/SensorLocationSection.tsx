import { useState } from 'react'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  AlertTitle,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  cn,
} from '@green-ecolution/ui'
import { ChevronDown, MapPin, MapPinOff } from 'lucide-react'
import MapPreview from '@/components/map-gl/MapPreview'
import SensorMarker from '@/components/map-gl/SensorMarker'
import type { Sensor } from '@/api/backendApi'

interface SensorLocationSectionProps {
  sensor: Sensor
}

const SensorLocationSection = ({ sensor }: SensorLocationSectionProps) => {
  const coord = sensor.coordinate
  const [showDetails, setShowDetails] = useState(false)

  return (
    <Card variant="outlined">
      <CardHeader>
        <div className="flex items-center justify-between gap-3">
          <div className="flex items-center gap-3">
            <div className="grid place-items-center size-9 rounded-lg bg-green-dark-50 text-green-dark">
              {coord ? <MapPin className="size-5" /> : <MapPinOff className="size-5" />}
            </div>
            <CardTitle>Standort</CardTitle>
          </div>
          {coord && (
            <button
              type="button"
              onClick={() => setShowDetails((v) => !v)}
              aria-expanded={showDetails}
              className="inline-flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-sm font-medium text-green-dark transition hover:bg-green-dark-50"
            >
              Koordinaten
              <ChevronDown
                className={cn('size-4 transition-transform', showDetails && 'rotate-180')}
              />
            </button>
          )}
        </div>
      </CardHeader>
      <CardContent>
        {coord ? (
          <div className="space-y-4">
            <MapPreview
              center={[coord.longitude, coord.latitude]}
              zoom={17}
              ariaLabel="Karte mit der Sensor-Position"
              className="h-72 sm:h-80"
            >
              <SensorMarker lng={coord.longitude} lat={coord.latitude} />
            </MapPreview>

            {showDetails && (
              <dl className="grid grid-cols-2 gap-x-6 gap-y-4 rounded-2xl border border-dark-100 bg-dark-50/40 p-5 animate-in fade-in slide-in-from-top-1">
                <div className="flex flex-col gap-1">
                  <dt className="text-xs uppercase tracking-widest text-muted-foreground">
                    Latitude
                  </dt>
                  <dd className="font-mono font-semibold text-base">
                    {coord.latitude.toFixed(6)}°
                  </dd>
                </div>
                <div className="flex flex-col gap-1">
                  <dt className="text-xs uppercase tracking-widest text-muted-foreground">
                    Longitude
                  </dt>
                  <dd className="font-mono font-semibold text-base">
                    {coord.longitude.toFixed(6)}°
                  </dd>
                </div>
                <p className="col-span-2 text-xs leading-relaxed text-muted-foreground">
                  Die Position wird vom verknüpften Baum übernommen.
                </p>
              </dl>
            )}
          </div>
        ) : (
          <Alert variant="warning" className="w-full">
            <div className="flex gap-3">
              <AlertIcon variant="warning" />
              <AlertContent>
                <AlertTitle>Sensor noch nicht im Feld</AlertTitle>
                <AlertDescription>
                  Dieser Sensor wurde noch nicht aktiviert und hat keinen Standort. Sobald er einem
                  Baum oder Beet zugeordnet wird, erscheint hier eine Karte.
                </AlertDescription>
              </AlertContent>
            </div>
          </Alert>
        )}
      </CardContent>
    </Card>
  )
}

export default SensorLocationSection
