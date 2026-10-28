import { Suspense, type ReactNode } from 'react'
import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { Trash2 } from 'lucide-react'
import type { SensorResponse } from '@green-ecolution/backend-client'
import MapPreview from '@/components/map-gl/MapPreview'
import SensorMarker from '@/components/map-gl/SensorMarker'
import PlacementRow from '@/components/sensor/detail/PlacementRow'
import { useMediaQuery } from '@/hooks/useMediaQuery'
import { wasteBinQueries } from '../../api/queries'
import { isBinTarget } from '../../lib/targets'
import FillStateBadge from '../FillStateBadge'

const BinRow = ({ binId }: { binId: string }) => {
  const { t } = useTranslation('waste')
  const { data: bin, isLoading, isError } = useQuery(wasteBinQueries.detail(binId))

  if (isLoading) {
    return <div className="mx-3 my-2.5 h-11 animate-pulse rounded-lg bg-dark-50" />
  }
  if (isError || !bin) {
    return (
      <PlacementRow
        icon={<Trash2 />}
        caption={t('sensor.linkedBin.loadFailedTitle')}
        title={t('sensor.linkedBin.loadFailedDescription', { id: binId })}
      />
    )
  }

  return (
    <PlacementRow
      icon={<Trash2 />}
      caption={t('sensor.placement.binLabel')}
      title={bin.number}
      meta={
        <>
          <FillStateBadge state={bin.fillState} percent={bin.fillPercent} />
          <span>{t('units.liters', { value: bin.volumeLiters })}</span>
        </>
      }
      link={{ to: '/waste-bins/$wasteBinId', params: { wasteBinId: bin.id } }}
    />
  )
}

const BinPlacement = ({ sensor, children }: { sensor: SensorResponse; children?: ReactNode }) => {
  const { t } = useTranslation('waste')
  const coord = sensor.location
  // On touch screens a pannable preview swallows the swipe meant to scroll the page.
  const mapInteractive = useMediaQuery('(pointer: fine)')
  const binId = isBinTarget(sensor.target) ? sensor.target.id : ''

  return (
    <>
      {coord && (
        <MapPreview
          center={[coord.longitude, coord.latitude]}
          zoom={17}
          interactive={mapInteractive}
          ariaLabel={t('sensor.placement.mapAriaLabel')}
          className="h-48 sm:h-56 xl:h-52"
        >
          <Suspense fallback={null}>
            <SensorMarker lng={coord.longitude} lat={coord.latitude} />
          </Suspense>
        </MapPreview>
      )}

      {children}

      <div className="divide-y divide-dark-100 overflow-hidden rounded-xl border border-dark-100">
        <BinRow binId={binId} />
      </div>
    </>
  )
}

export default BinPlacement
