import { useNavigate } from '@tanstack/react-router'
import { useMapEvents } from 'react-leaflet/hooks'
import useMapStore from '@/store/store'
import { useCallback, useEffect, useRef } from 'react'

const DEBOUNCE_MS = 150

const MapController = () => {
  const navigate = useNavigate()
  const setCenter = useMapStore((state) => state.map.setCenter)
  const setZoom = useMapStore((state) => state.map.setZoom)

  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  const scheduleUpdate = useCallback(
    (lat: number, lng: number, zoom: number) => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current)
      }
      timeoutRef.current = setTimeout(() => {
        setCenter([lat, lng])
        setZoom(zoom)
        navigate({
          to: '.',
          search: (prev) => ({ ...prev, lat, lng, zoom }),
          replace: true,
        }).catch((error) => console.error('Navigation failed:', error))
      }, DEBOUNCE_MS)
    },
    [navigate, setCenter, setZoom],
  )

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current)
      }
    }
  }, [])

  const map = useMapEvents({
    dragend: () => {
      const center = map.getCenter()
      scheduleUpdate(center.lat, center.lng, map.getZoom())
    },
    zoomend: () => {
      const center = map.getCenter()
      scheduleUpdate(center.lat, center.lng, map.getZoom())
    },
  })

  return null
}

export default MapController
