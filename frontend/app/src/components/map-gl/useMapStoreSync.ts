import { useEffect, useRef } from 'react'
import { useNavigate } from '@tanstack/react-router'
import useStore from '@/store/store'
import { useMaplibreMap } from './MapContext'

// The store updates on every move/zoom end (layers + the group↔tree threshold
// depend on it), but the URL search params only sync once the user has stopped
// moving, so rapid successive drags don't flood navigation/the route loader.
const URL_SYNC_DEBOUNCE_MS = 500

const useMapStoreSync = () => {
  const map = useMaplibreMap()
  const navigate = useNavigate()
  const setCenter = useStore((s) => s.setMapCenter)
  const setZoom = useStore((s) => s.setMapZoom)
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    const handler = () => {
      const c = map.getCenter()
      setCenter([c.lat, c.lng])
      setZoom(map.getZoom())

      if (timer.current) clearTimeout(timer.current)
      timer.current = setTimeout(() => {
        const center = map.getCenter()
        // The /map route validates `zoom` as an integer; MapLibre's zoom is
        // fractional, so round it or the param fails validation and resets to
        // mapMinZoom — which would clear the individual-tree layer.
        navigate({
          to: '.',
          search: (prev) => ({
            ...prev,
            lat: center.lat,
            lng: center.lng,
            zoom: Math.round(map.getZoom()),
          }),
          replace: true,
        }).catch((error) => console.error('Navigation failed:', error))
      }, URL_SYNC_DEBOUNCE_MS)
    }

    map.on('moveend', handler)
    map.on('zoomend', handler)
    return () => {
      if (timer.current) clearTimeout(timer.current)
      map.off('moveend', handler)
      map.off('zoomend', handler)
    }
  }, [map, navigate, setCenter, setZoom])
}

export default useMapStoreSync
