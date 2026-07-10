/* eslint-disable react-refresh/only-export-components */
import { useEffect, useRef } from 'react'
import { Marker } from 'maplibre-gl'
import type { RefillPointResponse, StartPointResponse } from '@green-ecolution/backend-client'
import { useMaplibreMap } from './MapContext'

export interface RoutePointMarkerData {
  lng: number
  lat: number
  name: string
  kind: 'start' | 'refill'
}

interface RoutePointMarkersProps {
  points: RoutePointMarkerData[]
}

// lucide "house" and "droplet" glyphs, inlined like SENSOR_ICON_SVG
const HOUSE_ICON_SVG = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" xmlns="http://www.w3.org/2000/svg"><path d="M15 21v-8a1 1 0 0 0-1-1h-4a1 1 0 0 0-1 1v8"/><path d="M3 10a2 2 0 0 1 .709-1.528l7-5.999a2 2 0 0 1 2.582 0l7 5.999A2 2 0 0 1 21 10v9a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/></svg>`
const DROPLET_ICON_SVG = `<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" xmlns="http://www.w3.org/2000/svg"><path d="M12 22a7 7 0 0 0 7-7c0-2-1-3.9-3-5.5s-3.5-4-4-6.5c-.5 2.5-2 4.9-4 6.5C6 11.1 5 13 5 15a7 7 0 0 0 7 7z"/></svg>`

// Resolves the depot the same way the backend does: by name, falling back to
// the default depot; a refill entry matching the start point is shown as start only.
export const buildRoutePoints = (
  startPoints: StartPointResponse[] | null | undefined,
  startPointName: string | null | undefined,
  refillPoints: RefillPointResponse[] | undefined,
): RoutePointMarkerData[] => {
  const startPoint =
    startPoints?.find((sp) => sp.name === startPointName) ?? startPoints?.find((sp) => sp.isDefault)
  const points: RoutePointMarkerData[] = []
  if (startPoint) {
    points.push({ lng: startPoint.lon, lat: startPoint.lat, name: startPoint.name, kind: 'start' })
  }
  for (const refill of refillPoints ?? []) {
    if (refill.name === startPoint?.name) continue
    points.push({ lng: refill.lon, lat: refill.lat, name: refill.name, kind: 'refill' })
  }
  return points
}

const buildPointElement = ({ name, kind }: RoutePointMarkerData) => {
  const el = document.createElement('div')
  el.className = `flex items-center gap-1 rounded-full border-2 border-white py-0.5 pl-1 pr-2.5 text-white shadow-[0_2px_6px_rgba(0,0,0,0.35)] ${
    kind === 'start' ? 'bg-green-dark' : 'bg-blue-600'
  }`
  const icon = document.createElement('span')
  icon.className = 'grid size-5 place-items-center'
  icon.innerHTML = kind === 'start' ? HOUSE_ICON_SVG : DROPLET_ICON_SVG
  const label = document.createElement('span')
  label.className = 'whitespace-nowrap text-xs font-semibold'
  label.textContent = `${kind === 'start' ? 'Startpunkt' : 'Nachfüllpunkt'} · ${name}`
  el.append(icon, label)
  return el
}

// Read-only chips marking the depot and the refill stations a route visits.
const RoutePointMarkers = ({ points }: RoutePointMarkersProps) => {
  const map = useMaplibreMap()
  const markersRef = useRef<Marker[]>([])

  useEffect(() => {
    markersRef.current = points.map((point) =>
      new Marker({ element: buildPointElement(point) })
        .setLngLat([point.lng, point.lat])
        .addTo(map),
    )
    return () => {
      markersRef.current.forEach((marker) => marker.remove())
      markersRef.current = []
    }
  }, [map, points])

  return null
}

export default RoutePointMarkers
