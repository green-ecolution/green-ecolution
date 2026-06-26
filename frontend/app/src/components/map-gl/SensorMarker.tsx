import { useEffect } from 'react'
import { Marker } from 'maplibre-gl'
import { useMaplibreMap } from './MapContext'

interface SensorMarkerProps {
  lng: number
  lat: number
}

// Read-only marker for a sensor's position; blue to stand apart from the green tree markers.
const SensorMarker = ({ lng, lat }: SensorMarkerProps) => {
  const map = useMaplibreMap()

  useEffect(() => {
    const marker = new Marker({ color: '#2563eb' }).setLngLat([lng, lat]).addTo(map)
    return () => {
      marker.remove()
    }
  }, [map, lng, lat])

  return null
}

export default SensorMarker
