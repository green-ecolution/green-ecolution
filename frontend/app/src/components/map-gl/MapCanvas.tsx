import 'maplibre-gl/dist/maplibre-gl.css'
import maplibregl, { type Map as MaplibreMap } from 'maplibre-gl'
import React, { useEffect, useRef, useState } from 'react'
import { useSuspenseQuery } from '@tanstack/react-query'
import { mapInfoQuery } from '@/api/queries'
import useStore from '@/store/store'
import { MapContext } from './MapContext'
import { OPENFREEMAP_STYLE_URL } from './mapStyle'

const MapCanvas = ({ children }: React.PropsWithChildren) => {
  const { data: mapInfo } = useSuspenseQuery(mapInfoQuery())
  const containerRef = useRef<HTMLDivElement>(null)
  const [map, setMap] = useState<MaplibreMap | null>(null)

  const [swLat, swLng, neLat, neLng] = mapInfo.bbox

  useEffect(() => {
    if (!containerRef.current) return
    const { mapCenter, mapZoom, mapMinZoom, mapMaxZoom } = useStore.getState()
    const m = new maplibregl.Map({
      container: containerRef.current,
      style: OPENFREEMAP_STYLE_URL,
      center: [mapCenter[1], mapCenter[0]],
      zoom: mapZoom,
      minZoom: mapMinZoom,
      maxZoom: mapMaxZoom,
      maxBounds: [
        [swLng, swLat],
        [neLng, neLat],
      ],
      attributionControl: { compact: true },
    })
    m.on('load', () => setMap(m))
    return () => {
      m.remove()
      setMap(null)
    }
  }, [swLat, swLng, neLat, neLng])

  return (
    <div className="absolute inset-0 flex flex-col">
      <div ref={containerRef} className="min-h-0 flex-1" />
      <MapContext value={map}>{map ? children : null}</MapContext>
    </div>
  )
}

export default MapCanvas
