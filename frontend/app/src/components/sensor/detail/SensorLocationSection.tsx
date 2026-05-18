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
} from '@green-ecolution/ui'
import { MapPin, MapPinOff } from 'lucide-react'
import LocationMapPreview from '@/components/geolocation/LocationMapPreview'
import type { Sensor } from '@/api/backendApi'

interface SensorLocationSectionProps {
  sensor: Sensor
}

const SensorLocationSection = ({ sensor }: SensorLocationSectionProps) => {
  const coord = sensor.coordinate

  return (
    <Card variant="outlined">
      <CardHeader>
        <div className="flex items-center gap-3">
          <div className="grid place-items-center size-9 rounded-lg bg-green-dark-50 text-green-dark">
            {coord ? <MapPin className="size-5" /> : <MapPinOff className="size-5" />}
          </div>
          <CardTitle>Standort</CardTitle>
        </div>
      </CardHeader>
      <CardContent>
        {coord ? (
          <div className="grid gap-4 md:grid-cols-[2fr_1fr] md:gap-6">
            <LocationMapPreview latitude={coord.latitude} longitude={coord.longitude} />
            <dl className="flex flex-col gap-4 text-sm">
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
              <p className="text-xs text-muted-foreground leading-relaxed">
                Die Position wird vom verknüpften Baum übernommen.
              </p>
            </dl>
          </div>
        ) : (
          <Alert variant="warning" className="w-full">
            <div className="flex gap-3">
              <AlertIcon variant="warning" />
              <AlertContent>
                <AlertTitle>Sensor noch nicht im Feld</AlertTitle>
                <AlertDescription>
                  Dieser Sensor wurde noch nicht aktiviert und hat keinen Standort. Sobald er
                  einem Baum oder Beet zugeordnet wird, erscheint hier eine Karte.
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
