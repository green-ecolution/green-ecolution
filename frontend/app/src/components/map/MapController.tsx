import { useNavigate } from '@tanstack/react-router'
import { useMapEvents } from 'react-leaflet/hooks'
import useMapStore from '@/store/store'
import { useCallback, useEffect, useRef } from 'react'

const MapController = () => {
  const navigate = useNavigate()
  const { setCenter, setZoom } = useMapStore((state) => ({
    setCenter: state.map.setCenter,
    setZoom: state.map.setZoom,
  }))

  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const debouncedNavigate = useCallback(
    (lat: number, lng: number, zoom: number) => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current)
      }
      timeoutRef.current = setTimeout(() => {
        navigate({
          to: '.',
          search: (prev) => ({ ...prev, lat, lng, zoom }),
          replace: true,
        }).catch((error) => console.error('Navigation failed:', error))
      }, 150)
    },
    [navigate],
  )

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current)
      }
    }
  }, [])

  const map = useMapEvents({
    moveend: () => {
      const center = map.getCenter()
      const zoom = map.getZoom()
      setCenter([center.lat, center.lng])
      setZoom(zoom)
      debouncedNavigate(center.lat, center.lng, zoom)
    },
  })

  return null
}

export default MapController
