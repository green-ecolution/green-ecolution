import { Minus, Plus } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useMaplibreMap } from './MapContext'
import MapControlButton from './MapControlButton'

const ZoomControls = () => {
  const { t } = useTranslation('map')
  const map = useMaplibreMap()
  return (
    <>
      <MapControlButton aria-label={t('controls.zoomInAriaLabel')} onClick={() => map.zoomIn()}>
        <Plus className="!size-6" />
      </MapControlButton>
      <MapControlButton aria-label={t('controls.zoomOutAriaLabel')} onClick={() => map.zoomOut()}>
        <Minus className="!size-6" />
      </MapControlButton>
    </>
  )
}

export default ZoomControls
