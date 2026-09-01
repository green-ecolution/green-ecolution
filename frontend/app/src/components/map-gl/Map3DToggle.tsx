import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useMaplibreMap } from './MapContext'
import MapControlButton from './MapControlButton'

const PITCH_3D = 60

const Map3DToggle = () => {
  const { t } = useTranslation('map')
  const map = useMaplibreMap()
  const [pitched, setPitched] = useState(() => map.getPitch() > 0)

  useEffect(() => {
    const onPitch = () => setPitched(map.getPitch() > 0)
    map.on('pitch', onPitch)
    return () => {
      map.off('pitch', onPitch)
    }
  }, [map])

  return (
    <MapControlButton
      active={pitched}
      aria-pressed={pitched}
      aria-label={pitched ? t('controls.switchTo2dAriaLabel') : t('controls.switchTo3dAriaLabel')}
      onClick={() => map.easeTo({ pitch: pitched ? 0 : PITCH_3D })}
    >
      <span className="text-sm font-bold">3D</span>
    </MapControlButton>
  )
}

export default Map3DToggle
