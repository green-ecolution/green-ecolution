import { useEffect, useState } from 'react'
import { useMaplibreMap } from './MapContext'
import MapControlButton from './MapControlButton'

const CompassButton = () => {
  const map = useMaplibreMap()
  const [bearing, setBearing] = useState(() => map.getBearing())
  const [pitched, setPitched] = useState(() => map.getPitch() > 0)

  useEffect(() => {
    const onRotate = () => setBearing(map.getBearing())
    const onPitch = () => setPitched(map.getPitch() > 0)
    map.on('rotate', onRotate)
    map.on('pitch', onPitch)
    return () => {
      map.off('rotate', onRotate)
      map.off('pitch', onPitch)
    }
  }, [map])

  const oriented = Math.abs(bearing) > 0.5 || pitched

  return (
    <MapControlButton
      aria-label="Karte nach Norden ausrichten"
      onClick={() => map.easeTo({ bearing: 0, pitch: 0 })}
      className={oriented ? 'ring-2 ring-green-dark/40' : undefined}
    >
      <svg
        data-testid="compass-needle"
        viewBox="0 0 24 24"
        className="!size-6"
        style={{ transform: `rotate(${-bearing}deg)` }}
        aria-hidden
      >
        <path d="M12 2 L15.2 12 H8.8 Z" fill="#E44E4D" />
        <path d="M12 22 L8.8 12 H15.2 Z" fill="#A2A2A2" />
        <circle cx="12" cy="12" r="1.6" fill="currentColor" />
      </svg>
    </MapControlButton>
  )
}

export default CompassButton
