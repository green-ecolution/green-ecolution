import { LocateFixed } from 'lucide-react'
import MapControlButton from './MapControlButton'
import { useGpsPosition } from './hooks/useGpsPosition'

const GpsButton = () => {
  const { active, toggle } = useGpsPosition()
  return (
    <MapControlButton
      aria-pressed={active}
      aria-label={active ? 'Standortanzeige beenden' : 'Eigenen Standort anzeigen'}
      onClick={toggle}
      className={active ? 'text-[#2563EB]' : undefined}
    >
      <LocateFixed className="!size-6" />
    </MapControlButton>
  )
}

export default GpsButton
