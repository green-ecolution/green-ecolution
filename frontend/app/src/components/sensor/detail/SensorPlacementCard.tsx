import { Suspense, useState, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertIcon,
  AlertTitle,
  Button,
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  cn,
} from '@green-ecolution/ui'
import { ChevronDown, Link2, MapPin } from 'lucide-react'
import MapPreview from '@/components/map-gl/MapPreview'
import SensorMarker from '@/components/map-gl/SensorMarker'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import type { Sensor } from '@/api/backendApi'
import { targetDescriptor, targetTextKey } from '@/lib/targets'
import { useActiveModules } from '@/modules/useActiveModules'
import PlacementRow from './PlacementRow'
import { useSensorActions } from './SensorActionsContext'
import { compactCardContent, compactCardHeader, compactCardTitle } from './cardDensity'

interface SensorPlacementCardProps {
  sensor: Sensor
}

/** A target whose module is off or unknown: the core knows only its label. */
const UnknownTargetPlacement = ({
  sensor,
  children,
}: SensorPlacementCardProps & { children?: ReactNode }) => {
  const { t } = useTranslation('sensor')
  const coord = sensor.location
  const mapInteractive = useMediaQuery('(pointer: fine)')

  return (
    <>
      {coord && (
        <MapPreview
          center={[coord.longitude, coord.latitude]}
          zoom={17}
          interactive={mapInteractive}
          ariaLabel={t('location.sensorMapAriaLabel')}
          className="h-48 sm:h-56 xl:h-52"
        >
          <SensorMarker lng={coord.longitude} lat={coord.latitude} />
        </MapPreview>
      )}
      {children}
      <div className="overflow-hidden rounded-xl border border-dark-100">
        <PlacementRow
          icon={<MapPin />}
          caption={t('placement.targetCaption')}
          title={sensor.targetLabel ?? t('placement.unnamedTarget')}
        />
      </div>
    </>
  )
}

const SensorPlacementCard = ({ sensor }: SensorPlacementCardProps) => {
  const { t, i18n } = useTranslation('sensor')
  const actions = useSensorActions()
  const { manifests } = useActiveModules()
  const coord = sensor.location
  const [showCoordinates, setShowCoordinates] = useState(false)
  const descriptor = targetDescriptor(manifests, sensor.target)
  const Placement = descriptor?.Placement
  const coordinates = coord && showCoordinates && (
    <div className="rounded-xl bg-dark-50 px-4 py-3 text-sm">
      <dl className="grid grid-cols-2 gap-x-6">
        <div>
          <dt className="text-muted-foreground">{t('location.latitudeLabel')}</dt>
          <dd className="font-mono font-semibold">{coord.latitude.toFixed(6)}°</dd>
        </div>
        <div>
          <dt className="text-muted-foreground">{t('location.longitudeLabel')}</dt>
          <dd className="font-mono font-semibold">{coord.longitude.toFixed(6)}°</dd>
        </div>
      </dl>
      <p className="mt-2 text-xs text-muted-foreground">{t('location.derivedFromTargetNotice')}</p>
    </div>
  )

  return (
    <Card variant="outlined" className="h-full">
      <CardHeader className={cn(compactCardHeader, 'flex-row items-center justify-between gap-3')}>
        <CardTitle className={compactCardTitle}>{t('placement.title')}</CardTitle>
        {coord && (
          <button
            type="button"
            onClick={() => setShowCoordinates((v) => !v)}
            aria-expanded={showCoordinates}
            className="-my-1.5 inline-flex min-h-9 items-center gap-1 rounded-lg px-2.5 text-sm font-medium text-green-dark transition hover:bg-green-dark-50"
          >
            {t('location.coordinatesToggle')}
            <ChevronDown
              className={cn(
                'size-4 transition-transform duration-base ease-out motion-reduce:transition-none',
                showCoordinates && 'rotate-180',
              )}
              aria-hidden
            />
          </button>
        )}
      </CardHeader>
      <CardContent className={cn(compactCardContent, 'flex flex-col gap-4')}>
        {!sensor.target ? (
          <Alert variant="warning" className="w-full">
            <div className="flex gap-3">
              <AlertIcon variant="warning" />
              <AlertContent>
                <AlertTitle>{t('placement.noTargetTitle')}</AlertTitle>
                <AlertDescription>{t('placement.noTargetDescription')}</AlertDescription>
                {actions.canAssign && (
                  <Button
                    variant="outline"
                    size="sm"
                    className="mt-3 gap-2 [&_svg]:size-4"
                    onClick={() => actions.requestActivate()}
                  >
                    <Link2 />
                    {i18n.t(
                      targetTextKey(
                        actions.textTarget,
                        'activateAction',
                        'sensor:actions.activateAssign',
                      ),
                    )}
                  </Button>
                )}
              </AlertContent>
            </div>
          </Alert>
        ) : (
          <>
            {Placement ? (
              <Suspense
                fallback={
                  <div className="h-48 rounded-xl bg-dark-50 animate-pulse sm:h-56 xl:h-52" />
                }
              >
                <Placement sensor={sensor}>{coordinates}</Placement>
              </Suspense>
            ) : (
              <UnknownTargetPlacement sensor={sensor}>{coordinates}</UnknownTargetPlacement>
            )}
          </>
        )}
      </CardContent>
    </Card>
  )
}

export default SensorPlacementCard
