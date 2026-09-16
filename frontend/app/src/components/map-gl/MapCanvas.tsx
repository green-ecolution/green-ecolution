import 'maplibre-gl/dist/maplibre-gl.css'
import { MaplibreMap } from './maplibre'
import React, { useEffect, useRef, useState } from 'react'
import useStore from '@/store/store'
import { DEFAULT_OPEN_ZOOM } from '@/lib/mapConfig'
import { MapContext } from './MapContext'
import { OPENFREEMAP_STYLE_URL } from './mapStyle'
import useMapView from './useMapView'

const MapCanvas = ({ children }: React.PropsWithChildren) => {
  const mapView = useMapView()
  const containerRef = useRef<HTMLDivElement>(null)
  const [map, setMap] = useState<MaplibreMap | null>(null)

  // Spread into the dependency list: a fresh object every render would rebuild
  // the map on every render.
  const [swLat, swLng, neLat, neLng] = mapView.bounds?.bbox ?? [null, null, null, null]
  const minZoom = mapView.bounds?.minZoom ?? null
  const maxZoom = mapView.bounds?.maxZoom ?? null
  const [viewLat, viewLng] = mapView.center

  useEffect(() => {
    if (!containerRef.current) return
    // A position in the store means the route carried one (a deep link) or the
    // user has already moved the map; otherwise the organization's viewport is
    // where this opens.
    const { mapCenter, mapZoom } = useStore.getState()
    const center = mapCenter ?? [viewLat, viewLng]
    const m = new MaplibreMap({
      container: containerRef.current,
      style: OPENFREEMAP_STYLE_URL,
      center: [center[1], center[0]],
      zoom: mapZoom ?? minZoom ?? DEFAULT_OPEN_ZOOM,
      // No limits means the organization lifted them; MapLibre wants the
      // options absent rather than a box spanning the world, so the map may
      // then be panned and zoomed as freely as the tiles allow.
      ...(swLat != null && swLng != null && neLat != null && neLng != null
        ? {
            maxBounds: [
              [swLng, swLat],
              [neLng, neLat],
            ] as [[number, number], [number, number]],
          }
        : {}),
      ...(minZoom != null ? { minZoom } : {}),
      ...(maxZoom != null ? { maxZoom } : {}),
      attributionControl: { compact: true },
    })
    m.on('load', () => setMap(m))
    return () => {
      m.remove()
      setMap(null)
    }
  }, [swLat, swLng, neLat, neLng, minZoom, maxZoom, viewLat, viewLng])

  return (
    <div className="absolute inset-0 flex flex-col">
      <div ref={containerRef} className="min-h-0 flex-1" />
      <MapContext value={map}>{map ? children : null}</MapContext>
    </div>
  )
}

export default MapCanvas
