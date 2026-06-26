import { useEffect, useRef } from 'react'
import { Marker } from 'maplibre-gl'
import { useMaplibreMap } from './MapContext'

interface SensorMarkerProps {
  lng: number
  lat: number
}

// Sensor "signal rings" glyph (from components/icons/Sensor) so the marker reads
// as a sensor chip rather than a generic pin.
const SENSOR_ICON_SVG = `<svg width="18" height="18" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg"><path fill-rule="evenodd" clip-rule="evenodd" d="M20.27 12C20.27 16.5674 16.5674 20.27 12 20.27C7.4326 20.27 3.73 16.5674 3.73 12C3.73 7.43261 7.4326 3.73 12 3.73C16.5674 3.73 20.27 7.43261 20.27 12ZM21.77 12C21.77 17.3958 17.3958 21.77 12 21.77C6.60417 21.77 2.23 17.3958 2.23 12C2.23 6.60418 6.60417 2.23 12 2.23C17.3958 2.23 21.77 6.60418 21.77 12ZM14.5701 12C14.5701 13.4207 13.419 14.5715 12 14.5715C10.581 14.5715 9.42986 13.4207 9.42986 12C9.42986 10.5793 10.581 9.4285 12 9.4285C13.419 9.4285 14.5701 10.5793 14.5701 12ZM16.0701 12C16.0701 14.2486 14.2479 16.0715 12 16.0715C9.75212 16.0715 7.92986 14.2486 7.92986 12C7.92986 9.75138 9.75212 7.9285 12 7.9285C14.2479 7.9285 16.0701 9.75138 16.0701 12Z" fill="currentColor"/></svg>`

// Read-only chip marking a sensor's position; green-dark badge to match the
// sensor accents used elsewhere in the app.
const SensorMarker = ({ lng, lat }: SensorMarkerProps) => {
  const map = useMaplibreMap()
  const markerRef = useRef<Marker | null>(null)

  useEffect(() => {
    const el = document.createElement('div')
    el.className =
      'grid size-8 place-items-center rounded-full border-2 border-white bg-green-dark text-white shadow-[0_2px_6px_rgba(0,0,0,0.35)]'
    el.innerHTML = SENSOR_ICON_SVG
    const marker = new Marker({ element: el }).setLngLat([lng, lat]).addTo(map)
    markerRef.current = marker
    return () => {
      marker.remove()
      markerRef.current = null
    }
    // Create the marker once; position updates happen in the effect below.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [map])

  useEffect(() => {
    markerRef.current?.setLngLat([lng, lat])
  }, [lng, lat])

  return null
}

export default SensorMarker
