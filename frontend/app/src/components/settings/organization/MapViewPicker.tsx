import { useCallback, useEffect, useRef, useState } from 'react'
import { Marker } from 'maplibre-gl'
import { useTranslation } from 'react-i18next'
import { Button } from '@green-ecolution/ui'
import type { MapViewDto } from '@/api/backendApi'
import type { MaplibreMap } from '@/components/map-gl/maplibre'
import MapPreview from '@/components/map-gl/MapPreview'
import { useMaplibreMap } from '@/components/map-gl/MapContext'

// lucide "map-pin", inlined like the other marker glyphs.
const PIN_ICON_SVG = `<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" xmlns="http://www.w3.org/2000/svg"><path d="M20 10c0 4.993-5.539 10.193-7.399 11.799a1 1 0 0 1-1.202 0C9.539 20.193 4 14.993 4 10a8 8 0 0 1 16 0"/><circle cx="12" cy="10" r="3"/></svg>`

const boundsOf = (map: MaplibreMap): [number, number, number, number] => {
  const bounds = map.getBounds()
  const sw = bounds.getSouthWest()
  const ne = bounds.getNorthEast()
  return [sw.lat, sw.lng, ne.lat, ne.lng]
}

/** Hands the map instance out of `MapPreview`'s context to the controls below it. */
const MapHandle = ({ onMap }: { onMap: (map: MaplibreMap) => void }) => {
  const map = useMaplibreMap()
  useEffect(() => {
    onMap(map)
  }, [map, onMap])
  return null
}

interface CentreMarkerProps {
  /** Where the pin starts. It is dragged afterwards, not re-positioned. */
  lat: number
  lng: number
  label: string
  onMove: (lat: number, lng: number) => void
}

const CentreMarker = ({ lat, lng, label, onMove }: CentreMarkerProps) => {
  const map = useMaplibreMap()
  const start = useRef<[number, number]>([lng, lat])

  useEffect(() => {
    const element = document.createElement('div')
    element.className =
      'grid size-8 cursor-grab place-items-center rounded-full border-2 border-white bg-green-dark text-white shadow-[0_2px_6px_rgba(0,0,0,0.35)] active:cursor-grabbing'
    element.innerHTML = PIN_ICON_SVG
    element.setAttribute('aria-label', label)
    element.title = label

    const marker = new Marker({ element, draggable: true }).setLngLat(start.current).addTo(map)
    marker.on('dragend', () => {
      const position = marker.getLngLat()
      onMove(position.lat, position.lng)
    })
    return () => {
      marker.remove()
    }
  }, [map, label, onMove])

  return null
}

interface MapViewPickerProps {
  value: MapViewDto
  onApply: (view: MapViewDto) => void
}

/**
 * Frames a viewport by hand: the visible extent becomes the bounding box the
 * map may pan in, the pin marks where it opens. Two gestures instead of four
 * coordinate fields.
 *
 * The pin lives in a ref, not in state: the marker draws itself, and nothing
 * on this screen has to re-render while it is being dragged. Should it end up
 * outside the chosen extent, the draft's own validation says so — the same
 * path that catches a water demand typed out of range.
 */
const MapViewPicker = ({ value, onApply }: MapViewPickerProps) => {
  const { t } = useTranslation('settings')
  const [map, setMap] = useState<MaplibreMap | null>(null)
  const pin = useRef<[number, number]>([value.center[0], value.center[1]])

  const handlePinMove = useCallback((lat: number, lng: number) => {
    pin.current = [lat, lng]
  }, [])

  // Without a limit the visible extent means nothing, so applying takes only
  // the marker and leaves the viewport unbounded.
  const restricted = value.bbox != null

  const handleApply = () => {
    if (!map) return
    const [lat, lng] = pin.current
    // The zoom range is edited in its own fields; applying re-frames the box
    // and the centre, and must not quietly reset what was typed there.
    onApply(
      restricted
        ? {
            center: [lat, lng],
            bbox: boundsOf(map),
            minZoom: value.minZoom,
            maxZoom: value.maxZoom,
          }
        : { center: [lat, lng], bbox: null, minZoom: undefined, maxZoom: undefined },
    )
  }

  // Framed once, at mount. While editing, the map is the input device: feeding
  // the extent it just produced back in would re-fit it with padding, and the
  // zoom would step out a notch on every apply.
  const [framing] = useState<[[number, number], [number, number]] | undefined>(() =>
    value.bbox
      ? [
          [value.bbox[1], value.bbox[0]],
          [value.bbox[3], value.bbox[2]],
        ]
      : undefined,
  )
  const [openAt] = useState<[number, number]>(() => [value.center[1], value.center[0]])

  return (
    <div className="flex flex-col gap-3">
      <MapPreview
        bounds={framing}
        center={framing ? undefined : openAt}
        zoom={12}
        interactive
        className="h-80"
        ariaLabel={t('organization.settings.mapView.pickerAriaLabel')}
      >
        <MapHandle onMap={setMap} />
        <CentreMarker
          lat={value.center[0]}
          lng={value.center[1]}
          label={t('organization.settings.mapView.pinAriaLabel')}
          onMove={handlePinMove}
        />
      </MapPreview>

      <p className="max-w-[62ch] text-sm text-dark-600">
        {restricted
          ? t('organization.settings.mapView.pickerHint')
          : t('organization.settings.mapView.pickerHintUnrestricted')}
      </p>

      <Button
        type="button"
        variant="outline"
        size="sm"
        className="w-fit"
        disabled={!map}
        onClick={handleApply}
      >
        {restricted
          ? t('organization.settings.mapView.apply')
          : t('organization.settings.mapView.applyCentre')}
      </Button>
    </div>
  )
}

export default MapViewPicker
