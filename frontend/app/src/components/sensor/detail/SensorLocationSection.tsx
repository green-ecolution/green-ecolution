import { Suspense, useMemo, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
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
import { treeQueries } from '@/api/queries'
import MapPreview from '@/components/map-gl/MapPreview'
import SensorMarker from '@/components/map-gl/SensorMarker'
import useViewportBBox from '@/components/map-gl/hooks/useViewportBBox'
import useTreeMarkerLayer, {
  type TreeMarkerPoint,
} from '@/components/map-gl/layers/useTreeMarkerLayer'
import useClusterBoundaryLayer from '@/components/map-gl/layers/useClusterBoundaryLayer'
import type { Sensor } from '@/api/backendApi'

interface SensorLocationSectionProps {
  sensor: Sensor
}

const LocationTreeLayer = () => {
  const bbox = useViewportBBox()
  const { data } = useQuery(treeQueries.markers({ bbox }))
  const trees = useMemo<TreeMarkerPoint[]>(
    () =>
      (data?.data ?? []).map((t) => ({
        id: t.id,
        longitude: t.longitude,
        latitude: t.latitude,
        status: t.wateringStatus,
      })),
    [data],
  )
  useTreeMarkerLayer({
    trees,
    sourceId: 'gec-location-trees',
    circleLayerId: 'gec-location-tree-circle',
    iconLayerId: 'gec-location-tree-icon',
    interactive: false,
  })
  return null
}

const LocationClusterBoundaries = () => {
  useClusterBoundaryLayer({ interactive: false })
  return null
}

const SensorLocationSection = ({ sensor }: SensorLocationSectionProps) => {
  const { t } = useTranslation('sensor')
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
            <CardTitle>{t('location.title')}</CardTitle>
          </div>
          {coord && (
            <button
              type="button"
              onClick={() => setShowDetails((v) => !v)}
              aria-expanded={showDetails}
              className="inline-flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-sm font-medium text-green-dark transition hover:bg-green-dark-50"
            >
              {t('location.coordinatesToggle')}
              <ChevronDown
                className={cn(
                  'size-4 transition-transform duration-base ease-out motion-reduce:transition-none',
                  showDetails && 'rotate-180',
                )}
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
              interactive
              ariaLabel={t('location.mapAriaLabel')}
              className="h-72 sm:h-80"
            >
              <Suspense fallback={null}>
                <LocationClusterBoundaries />
              </Suspense>
              <LocationTreeLayer />
              <SensorMarker lng={coord.longitude} lat={coord.latitude} />
            </MapPreview>

            {showDetails && (
              <dl className="grid grid-cols-2 gap-x-6 gap-y-4 rounded-2xl border border-dark-100 bg-dark-50/40 p-5 animate-in fade-in slide-in-from-top-1">
                <div className="flex flex-col gap-1">
                  <dt className="text-xs uppercase tracking-widest text-muted-foreground">
                    {t('location.latitudeLabel')}
                  </dt>
                  <dd className="font-mono font-semibold text-base">
                    {coord.latitude.toFixed(6)}°
                  </dd>
                </div>
                <div className="flex flex-col gap-1">
                  <dt className="text-xs uppercase tracking-widest text-muted-foreground">
                    {t('location.longitudeLabel')}
                  </dt>
                  <dd className="font-mono font-semibold text-base">
                    {coord.longitude.toFixed(6)}°
                  </dd>
                </div>
                <p className="col-span-2 text-xs leading-relaxed text-muted-foreground">
                  {t('location.derivedFromTreeNotice')}
                </p>
              </dl>
            )}
          </div>
        ) : (
          <Alert variant="warning" className="w-full">
            <div className="flex gap-3">
              <AlertIcon variant="warning" />
              <AlertContent>
                <AlertTitle>{t('location.notInFieldTitle')}</AlertTitle>
                <AlertDescription>{t('location.notInFieldDescription')}</AlertDescription>
              </AlertContent>
            </div>
          </Alert>
        )}
      </CardContent>
    </Card>
  )
}

export default SensorLocationSection
