import defaultIconPng from 'leaflet/dist/images/marker-icon.png'
import L, { DivIcon, Icon, IconOptions } from 'leaflet'
import { Marker } from 'react-leaflet'
import { Marker as LeafletMarker } from 'leaflet'
import { useMemo, useRef } from 'react'

const defaultIcon = new Icon({
  iconUrl: defaultIconPng,
  iconSize: [25, 41],
  iconAnchor: [12, 41],
  popupAnchor: [1, -34],
  shadowSize: [41, 41],
})

export interface MapMarkerProps {
  icon: Icon<IconOptions> | DivIcon | undefined
  position: [number, number]
  onClick: () => void
}

const MapMarker = ({ position, icon, onClick }: MapMarkerProps) => {
  return (
    <Marker
      position={position}
      icon={icon}
      eventHandlers={{
        click: () => onClick(),
      }}
    ></Marker>
  )
}

interface DragableMarkerProps {
  position: L.LatLng
  onMove?: (latlng: L.LatLng) => void
  onDrag?: (latlng: L.LatLng) => void
}

export const DragableMarker = ({ position, onDrag, onMove }: DragableMarkerProps) => {
  const markerRef = useRef<LeafletMarker>(null)
  const eventHandlers = useMemo(
    () => ({
      move() {
        const marker = markerRef.current
        if (marker != null) {
          onMove?.(marker.getLatLng())
        }
      },
      dragend() {
        const marker = markerRef.current
        if (marker != null) {
          onDrag?.(marker.getLatLng())
        }
      },
    }),
    [onDrag, onMove],
  )

  return (
    <Marker
      eventHandlers={eventHandlers}
      icon={defaultIcon}
      ref={markerRef}
      draggable
      position={position}
    />
  )
}

export default MapMarker
