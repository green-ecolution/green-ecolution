import 'maplibre-gl/dist/maplibre-gl.css'
import maplibregl, { type LngLatBoundsLike, type Map as MaplibreMap } from 'maplibre-gl'
import { type PropsWithChildren, useEffect, useRef, useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { cn } from '@green-ecolution/ui'
import { mapInfoQuery } from '@/api/queries'
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

interface MapPreviewCanvasProps extends MapPreviewProps {
  fallbackCenter?: [number, number]
}

const FIT_OPTIONS = { padding: 48, maxZoom: 18 } as const

const frameClasses = (className?: string) =>
  cn('relative w-full overflow-hidden rounded-2xl border border-dark-100 shadow-cards', className)

const MapPreviewCanvas = ({
  center,
  zoom = 17,
  bounds,
  interactive = false,
  className,
  ariaLabel,
  fallbackCenter,
  children,
}: MapPreviewCanvasProps) => {
  const containerRef = useRef<HTMLDivElement>(null)
  const [map, setMap] = useState<MaplibreMap | null>(null)
  // Construction-time options; later changes are handled by the effects below.
  const initialOptions = useRef({ center, zoom, bounds, interactive, fallbackCenter })

  useEffect(() => {
    if (!containerRef.current) return
    const { center, zoom, bounds, interactive, fallbackCenter } = initialOptions.current
    // Frame the map at construction time — a post-load fitBounds would visibly
    // fly in from the fallback center.
    const framing = bounds
      ? { bounds, fitBoundsOptions: FIT_OPTIONS }
      : { center: center ?? fallbackCenter ?? ([0, 0] as [number, number]), zoom }
    const m = new maplibregl.Map({
      container: containerRef.current,
      style: OPENFREEMAP_STYLE_URL,
      interactive,
      attributionControl: { compact: true },
      ...framing,
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
  }, [])

  // Only react to an actual bounds change, never on first run: the initial
  // bounds are applied at construction.
  const appliedBoundsRef = useRef(bounds)
  useEffect(() => {
    if (!map || !bounds) return
    if (JSON.stringify(appliedBoundsRef.current) === JSON.stringify(bounds)) return
    appliedBoundsRef.current = bounds
    map.fitBounds(bounds, FIT_OPTIONS)
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
      className={frameClasses(className)}
    >
      <div className="absolute inset-0 flex flex-col">
        <div ref={containerRef} className="min-h-0 flex-1" />
      </div>
      <MapContext value={map}>{map ? children : null}</MapContext>
    </div>
  )
}

// Standalone read-only MapLibre instance for inline previews (sensor detail,
// onboarding wizard); non-interactive by default. Children rendered via
// MapContext draw markers/layers. Without explicit center/bounds the map
// frames the municipality center from the /info/map endpoint, like the main map.
const MapPreview = (props: MapPreviewProps) => {
  const { data: mapInfo, isError } = useQuery({
    ...mapInfoQuery(),
    enabled: !props.center && !props.bounds,
  })
  const needsFallback = !props.center && !props.bounds

  if (needsFallback && !mapInfo && !isError) {
    return <div aria-label={props.ariaLabel} className={frameClasses(props.className)} />
  }

  return (
    <MapPreviewCanvas
      {...props}
      fallbackCenter={mapInfo ? [mapInfo.center[1], mapInfo.center[0]] : undefined}
    />
  )
}

export default MapPreview
