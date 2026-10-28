import { useMemo } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Loader2, MapPin } from 'lucide-react'
import { AccuracyBadge, Button } from '@green-ecolution/ui'
import GeolocationPermissionNotice from '@/components/geolocation/GeolocationPermissionNotice'
import LocationMapPreview from '@/components/geolocation/LocationMapPreview'
import type { TargetWizardStepProps } from '@/modules/types'
import { wasteBinQueries } from '../../../api/queries'
import { distanceMeters } from '../../../lib/geo'
import { isBinTarget } from '../../../lib/targets'
import BinPickList from '../BinPickList'
import { binSelection } from '../useBinSelection'

const SensorBinStep = ({
  current,
  onSelect,
  position,
  geolocationStatus: status,
  geolocationError: errorMessage,
  onRelocate,
}: TargetWizardStepProps) => {
  const { t } = useTranslation('waste')
  const { data, isLoading, isError, refetch } = useQuery(wasteBinQueries.markers())
  const selectedId = isBinTarget(current) ? current.id : null

  const bins = useMemo(() => {
    const all = data ?? []
    if (!position) return all
    const distance = (bin: (typeof all)[number]) => distanceMeters(position, bin)
    return all.slice().sort((a, b) => distance(a) - distance(b))
  }, [data, position])

  const gpsNotice: 'denied' | 'unsupported' | 'error' | null =
    status === 'denied' || status === 'unsupported' || status === 'error' ? status : null
  const gpsPending = !position && !gpsNotice

  return (
    <div className="space-y-6">
      <header className="space-y-2">
        <h1 className="font-lato text-3xl font-bold lg:text-4xl">
          {t('sensor.wizard.bin.header.title')}
        </h1>
        <p className="max-w-prose text-sm text-muted-foreground">
          {t('sensor.wizard.bin.header.description')}
        </p>
      </header>

      {position && (
        <div className="flex items-center gap-3 text-sm">
          <AccuracyBadge accuracyMeters={position.accuracy} />
          <Button variant="link" size="sm" className="h-auto px-0" onClick={onRelocate}>
            <MapPin className="size-4" />
            {t('sensor.wizard.bin.relocateButton')}
          </Button>
        </div>
      )}

      {gpsPending && (
        <div
          role="status"
          aria-live="polite"
          className="flex items-center gap-3 text-sm text-muted-foreground"
        >
          <Loader2 className="size-5 animate-spin" aria-hidden />
          {t('sensor.wizard.bin.locatingNotice')}
        </div>
      )}

      {gpsNotice && (
        <GeolocationPermissionNotice
          status={gpsNotice}
          errorMessage={errorMessage}
          onRetry={gpsNotice === 'unsupported' ? undefined : onRelocate}
        />
      )}

      {position && (
        <LocationMapPreview
          latitude={position.latitude}
          longitude={position.longitude}
          accuracyMeters={position.accuracy}
        />
      )}

      <div className="flex max-h-[28rem] flex-col">
        <BinPickList
          bins={bins}
          loading={isLoading}
          failed={isError}
          onRetry={() => void refetch()}
          selectedId={selectedId}
          onSelect={(bin) => onSelect(binSelection(bin))}
        />
      </div>
    </div>
  )
}

export default SensorBinStep
