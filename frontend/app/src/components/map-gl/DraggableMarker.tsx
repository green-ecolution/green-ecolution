import { useEffect, useRef } from 'react'
import { Marker } from 'maplibre-gl'
import { useMaplibreMap } from './MapContext'

export interface DraggableMarkerLngLat {
  lng: number
  lat: number
}

interface DraggableMarkerProps {
  lng: number
  lat: number
  onDragEnd?: (pos: DraggableMarkerLngLat) => void
  onDrag?: (pos: DraggableMarkerLngLat) => void
}

const DraggableMarker = ({ lng, lat, onDragEnd, onDrag }: DraggableMarkerProps) => {
  const map = useMaplibreMap()
  const markerRef = useRef<Marker | null>(null)
  const onDragEndRef = useRef(onDragEnd)
  const onDragRef = useRef(onDrag)
  useEffect(() => {
    onDragEndRef.current = onDragEnd
    onDragRef.current = onDrag
  })

  useEffect(() => {
    const marker = new Marker({ draggable: true, color: '#486725' })
      .setLngLat([lng, lat])
      .addTo(map)
    marker.on('drag', () => {
      const p = marker.getLngLat()
      onDragRef.current?.({ lng: p.lng, lat: p.lat })
    })
    marker.on('dragend', () => {
      const p = marker.getLngLat()
      onDragEndRef.current?.({ lng: p.lng, lat: p.lat })
    })
    markerRef.current = marker
    return () => {
      marker.remove()
      markerRef.current = null
    }
    // Create the marker once; controlled position updates happen in the effect below.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [map])

  useEffect(() => {
    markerRef.current?.setLngLat([lng, lat])
  }, [lng, lat])

  return null
}

export default DraggableMarker
