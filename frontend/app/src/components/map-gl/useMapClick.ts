import { useEffect, useRef } from 'react'
import type { MapMouseEvent } from 'maplibre-gl'
import { useMaplibreMap } from './MapContext'

export interface MapClickLngLat {
  lng: number
  lat: number
}

export function useMapClick(onClick: (lngLat: MapClickLngLat) => void) {
  const map = useMaplibreMap()
  const cb = useRef(onClick)
  useEffect(() => {
    cb.current = onClick
  })

  useEffect(() => {
    const handler = (e: MapMouseEvent) => {
      cb.current({ lng: e.lngLat.lng, lat: e.lngLat.lat })
    }
    map.on('click', handler)
    return () => {
      map.off('click', handler)
    }
  }, [map])
}
