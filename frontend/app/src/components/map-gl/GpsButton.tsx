import { LocateFixed } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import MapControlButton from './MapControlButton'
import { GPS_BLUE, useGpsPosition } from './hooks/useGpsPosition'

const GpsButton = () => {
  const { t } = useTranslation('map')
  const { active, toggle } = useGpsPosition()
  return (
    <MapControlButton
      aria-pressed={active}
      aria-label={active ? t('controls.gpsStopAriaLabel') : t('controls.gpsShowAriaLabel')}
      onClick={toggle}
      style={active ? { color: GPS_BLUE } : undefined}
    >
      <LocateFixed className="!size-6" />
    </MapControlButton>
  )
}

export default GpsButton
