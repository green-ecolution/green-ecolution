import { LocateFixed } from 'lucide-react'
import MapControlButton from './MapControlButton'
import { GPS_BLUE, useGpsPosition } from './hooks/useGpsPosition'

const GpsButton = () => {
  const { active, toggle } = useGpsPosition()
  return (
    <MapControlButton
      aria-pressed={active}
      aria-label={active ? 'Standortanzeige beenden' : 'Eigenen Standort anzeigen'}
      onClick={toggle}
      style={active ? { color: GPS_BLUE } : undefined}
    >
      <LocateFixed className="!size-6" />
    </MapControlButton>
  )
}

export default GpsButton
