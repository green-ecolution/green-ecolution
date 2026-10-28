import { useState } from 'react'
import { MapPin } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import DraggableMarker from '@/components/map-gl/DraggableMarker'
import MapPreview from '@/components/map-gl/MapPreview'
import { useMapClick } from '@/components/map-gl/useMapClick'

export interface WasteBinPosition {
  lat: number
  lng: number
}

interface WasteBinPositionPickerProps {
  lat?: number
  lng?: number
  onChange: (position: WasteBinPosition) => void
  error?: string
}

// Only the first click places the bin; afterwards it moves by dragging the
// marker, so a stray click while panning the map cannot relocate it.
const FirstClickPlacement = ({
  placed,
  onPlace,
}: {
  placed: boolean
  onPlace: (position: WasteBinPosition) => void
}) => {
  useMapClick((position) => {
    if (!placed) onPlace(position)
  })
  return null
}

const WasteBinPositionPicker = ({ lat, lng, onChange, error }: WasteBinPositionPickerProps) => {
  const { t } = useTranslation('waste')
  const position = lat != null && lng != null ? { lat, lng } : undefined
  // Framed once: following every drag would pull the map from under the cursor.
  const [initialCenter] = useState<[number, number] | undefined>(() =>
    position ? [position.lng, position.lat] : undefined,
  )

  return (
    <div className="flex flex-col gap-2">
      <span className="text-sm font-medium">{t('form.positionLabel')}</span>
      <MapPreview
        interactive
        center={initialCenter}
        zoom={initialCenter ? 17 : 13}
        ariaLabel={t('form.mapAriaLabel')}
        className="h-72"
      >
        <FirstClickPlacement placed={position !== undefined} onPlace={onChange} />
        {position && <DraggableMarker lat={position.lat} lng={position.lng} onDragEnd={onChange} />}
      </MapPreview>
      {position && (
        <p className="flex items-center gap-2 text-sm tabular-nums text-dark-600">
          <MapPin aria-hidden className="size-4 text-primary-500" />
          {position.lat.toFixed(5)}, {position.lng.toFixed(5)}
        </p>
      )}
      {error ? (
        <p role="alert" className="text-sm text-destructive">
          {error}
        </p>
      ) : (
        <p className="text-sm text-dark-600">
          {position ? t('form.positionMoveHint') : t('form.positionHint')}
        </p>
      )}
    </div>
  )
}

export default WasteBinPositionPicker
