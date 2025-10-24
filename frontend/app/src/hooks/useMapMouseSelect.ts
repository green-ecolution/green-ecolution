import { LatLng, LeafletMouseEvent } from 'leaflet'
import { useMapEvents } from 'react-leaflet'

export function useMapMouseSelect(fn: (latlng: LatLng, e: LeafletMouseEvent) => void) {
  useMapEvents({
    click(e) {
      fn(e.latlng, e)
    },
  })
}
