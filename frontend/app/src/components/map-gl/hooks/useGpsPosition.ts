import { useCallback, useEffect, useRef, useState } from 'react'
import type { GeoJSONSource, Map as MaplibreMap } from 'maplibre-gl'
import { toast } from '@green-ecolution/ui'
import { useMaplibreMap } from '../MapContext'
import { metersCircle } from '../metersCircle'
import { isMapAlive } from '../mapReady'

const ACCURACY_SOURCE = 'gps-accuracy'
const ACCURACY_FILL = 'gps-accuracy-fill'
const ACCURACY_LINE = 'gps-accuracy-line'
const DOT_SOURCE = 'gps-dot'
const DOT_LAYER = 'gps-dot-layer'
const GPS_BLUE = '#2563EB'
const CENTER_ZOOM = 16

const ensureLayers = (map: MaplibreMap) => {
  if (map.getSource(ACCURACY_SOURCE)) return
  map.addSource(ACCURACY_SOURCE, {
    type: 'geojson',
    data: { type: 'FeatureCollection', features: [] },
  })
  map.addLayer({
    id: ACCURACY_FILL,
    type: 'fill',
    source: ACCURACY_SOURCE,
    paint: { 'fill-color': GPS_BLUE, 'fill-opacity': 0.12 },
  })
  map.addLayer({
    id: ACCURACY_LINE,
    type: 'line',
    source: ACCURACY_SOURCE,
    paint: { 'line-color': GPS_BLUE, 'line-width': 1.5 },
  })
  map.addSource(DOT_SOURCE, {
    type: 'geojson',
    data: { type: 'FeatureCollection', features: [] },
  })
  map.addLayer({
    id: DOT_LAYER,
    type: 'circle',
    source: DOT_SOURCE,
    paint: {
      'circle-radius': 7,
      'circle-color': GPS_BLUE,
      'circle-stroke-width': 2.5,
      'circle-stroke-color': '#FFFFFF',
    },
  })
}

export const useGpsPosition = () => {
  const map = useMaplibreMap()
  const [active, setActive] = useState(false)
  const watchId = useRef<number | null>(null)
  const centered = useRef(false)

  const stop = useCallback(() => {
    if (watchId.current !== null) {
      navigator.geolocation.clearWatch(watchId.current)
      watchId.current = null
    }
    centered.current = false
    if (isMapAlive(map)) {
      for (const id of [DOT_LAYER, ACCURACY_LINE, ACCURACY_FILL]) {
        if (map.getLayer(id)) map.removeLayer(id)
      }
      for (const id of [DOT_SOURCE, ACCURACY_SOURCE]) {
        if (map.getSource(id)) map.removeSource(id)
      }
    }
    setActive(false)
  }, [map])

  const handlePosition = useCallback(
    (pos: GeolocationPosition) => {
      if (!isMapAlive(map)) return
      const { longitude, latitude, accuracy } = pos.coords
      const bounds = map.getMaxBounds()
      if (bounds && !bounds.contains([longitude, latitude])) {
        toast.info('Position außerhalb des Kartenbereichs')
        stop()
        return
      }
      ensureLayers(map)
      map.getSource<GeoJSONSource>(ACCURACY_SOURCE)?.setData({
        type: 'FeatureCollection',
        features: accuracy > 0 ? [metersCircle(longitude, latitude, accuracy)] : [],
      })
      map.getSource<GeoJSONSource>(DOT_SOURCE)?.setData({
        type: 'FeatureCollection',
        features: [
          {
            type: 'Feature',
            geometry: { type: 'Point', coordinates: [longitude, latitude] },
            properties: {},
          },
        ],
      })
      if (!centered.current) {
        centered.current = true
        map.easeTo({
          center: [longitude, latitude],
          zoom: Math.max(map.getZoom(), CENTER_ZOOM),
        })
      }
    },
    [map, stop],
  )

  const start = useCallback(() => {
    if (!('geolocation' in navigator)) {
      toast.error('Standortbestimmung wird von diesem Browser nicht unterstützt')
      return
    }
    setActive(true)
    watchId.current = navigator.geolocation.watchPosition(
      handlePosition,
      (err) => {
        toast.error(
          err.code === err.PERMISSION_DENIED
            ? 'Standortzugriff verweigert'
            : 'Standort konnte nicht ermittelt werden',
        )
        stop()
      },
      { enableHighAccuracy: true },
    )
  }, [handlePosition, stop])

  useEffect(() => stop, [stop])

  const toggle = useCallback(() => {
    if (active) {
      stop()
    } else {
      start()
    }
  }, [active, start, stop])

  return { active, toggle }
}
