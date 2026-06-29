import { useEffect } from 'react'
import type { GeoJSONSource, Map as MaplibreMap } from 'maplibre-gl'
import { metersCircle } from '../metersCircle'
import { isMapAlive } from '../mapReady'

// idPrefix must be unique per ring; two rings sharing a prefix clash on the same source/layer ids.
export const useAccuracyRing = (
  map: MaplibreMap,
  idPrefix: string,
  longitude: number,
  latitude: number,
  accuracyMeters?: number | null,
) => {
  const sourceId = `${idPrefix}-accuracy`
  const fillId = `${idPrefix}-accuracy-fill`
  const lineId = `${idPrefix}-accuracy-line`

  useEffect(() => {
    if (!map.getSource(sourceId)) {
      map.addSource(sourceId, {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: [] },
      })
      map.addLayer({
        id: fillId,
        type: 'fill',
        source: sourceId,
        paint: { 'fill-color': '#486725', 'fill-opacity': 0.15 },
      })
      map.addLayer({
        id: lineId,
        type: 'line',
        source: sourceId,
        paint: { 'line-color': '#486725', 'line-width': 1.5 },
      })
    }
    return () => {
      if (!isMapAlive(map)) return
      for (const id of [lineId, fillId]) {
        if (map.getLayer(id)) map.removeLayer(id)
      }
      if (map.getSource(sourceId)) map.removeSource(sourceId)
    }
  }, [map, sourceId, fillId, lineId])

  useEffect(() => {
    if (!isMapAlive(map)) return
    const features =
      accuracyMeters && accuracyMeters > 0
        ? [metersCircle(longitude, latitude, accuracyMeters)]
        : []
    map.getSource<GeoJSONSource>(sourceId)?.setData({ type: 'FeatureCollection', features })
  }, [map, sourceId, longitude, latitude, accuracyMeters])
}
