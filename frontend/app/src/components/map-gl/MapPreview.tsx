import 'maplibre-gl/dist/maplibre-gl.css'
import maplibregl, { type LngLatBoundsLike, type Map as MaplibreMap } from 'maplibre-gl'
import { type PropsWithChildren, useEffect, useRef, useState } from 'react'
import { cn } from '@green-ecolution/ui'
import { MapContext } from './MapContext'
import { OPENFREEMAP_STYLE_URL } from './mapStyle'

interface MapPreviewProps extends PropsWithChildren {
  center?: [number, number]
  zoom?: number
  bounds?: LngLatBoundsLike
  interactive?: boolean
  className?: string
  ariaLabel?: string
}

const FIT_OPTIONS = { padding: 48, maxZoom: 18 } as const

// Standalone read-only MapLibre instance for inline previews (sensor detail,
// onboarding wizard); non-interactive by default. Children rendered via
// MapContext draw markers/layers.
const MapPreview = ({
  center,
  zoom = 17,
  bounds,
  interactive = false,
  className,
  ariaLabel,
  children,
}: MapPreviewProps) => {
  const containerRef = useRef<HTMLDivElement>(null)
  const [map, setMap] = useState<MaplibreMap | null>(null)

  useEffect(() => {
    if (!containerRef.current) return
    const m = new maplibregl.Map({
      container: containerRef.current,
      style: OPENFREEMAP_STYLE_URL,
      interactive,
      center: center ?? [0, 0],
      zoom,
      attributionControl: { compact: true },
    })
    // Under React StrictMode the map is created, torn down and recreated in the
    // same container; the surviving instance can stay blank until it is told to
    // recompute its size and repaint.
    m.on('load', () => {
      m.resize()
      m.triggerRepaint()
      setMap(m)
    })
    return () => {
      m.remove()
      setMap(null)
    }
    // Created once; controlled position updates happen in the effect below.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [])

  useEffect(() => {
    if (!map) return
    if (bounds) map.fitBounds(bounds, FIT_OPTIONS)
  }, [map, bounds])

  // Only react to an actual center change, never on first run: the initial center
  // is set at construction and children may frame the map imperatively (fitBounds).
  const appliedCenterRef = useRef(center)
  useEffect(() => {
    if (!map || bounds || !center) return
    const prev = appliedCenterRef.current
    if (prev?.[0] === center[0] && prev?.[1] === center[1]) return
    appliedCenterRef.current = center
    map.setCenter(center)
  }, [map, bounds, center])

  return (
    <div
      role={interactive ? undefined : 'img'}
      aria-label={ariaLabel}
      className={cn(
        'relative w-full overflow-hidden rounded-2xl border border-dark-100 shadow-cards',
        className,
      )}
    >
      <div className="absolute inset-0 flex flex-col">
        <div ref={containerRef} className="min-h-0 flex-1" />
      </div>
      <MapContext value={map}>{map ? children : null}</MapContext>
    </div>
  )
}

export default MapPreview
