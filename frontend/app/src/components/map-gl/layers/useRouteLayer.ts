import { useEffect } from 'react'
import type { GeoJSONSource } from 'maplibre-gl'
import type { Feature, LineString } from 'geojson'
import { useMaplibreMap } from '../MapContext'
import { LAYERS, ROUTE_COLORS, SOURCES } from '../mapStyle'
import { isMapAlive } from '../mapReady'

export interface UseRouteLayerOptions {
  /** GeoJSON positions ([lng, lat]); undefined/empty hides the route. */
  coordinates?: [number, number][]
}

const useRouteLayer = ({ coordinates }: UseRouteLayerOptions) => {
  const map = useMaplibreMap()

  useEffect(() => {
    if (!map.getSource(SOURCES.route)) {
      map.addSource(SOURCES.route, {
        type: 'geojson',
        data: { type: 'FeatureCollection', features: [] },
      })
    }
    if (!map.getLayer(LAYERS.routeCasing)) {
      map.addLayer({
        id: LAYERS.routeCasing,
        type: 'line',
        source: SOURCES.route,
        paint: { 'line-color': ROUTE_COLORS.casing, 'line-width': 7, 'line-opacity': 0.9 },
        layout: { 'line-join': 'round', 'line-cap': 'round' },
      })
    }
    if (!map.getLayer(LAYERS.routeLine)) {
      map.addLayer({
        id: LAYERS.routeLine,
        type: 'line',
        source: SOURCES.route,
        paint: { 'line-color': ROUTE_COLORS.line, 'line-width': 4 },
        layout: { 'line-join': 'round', 'line-cap': 'round' },
      })
    }
    return () => {
      if (!isMapAlive(map)) return
      for (const id of [LAYERS.routeLine, LAYERS.routeCasing]) {
        if (map.getLayer(id)) map.removeLayer(id)
      }
      if (map.getSource(SOURCES.route)) map.removeSource(SOURCES.route)
    }
  }, [map])

  useEffect(() => {
    if (!isMapAlive(map)) return
    const feature: Feature<LineString> = {
      type: 'Feature',
      geometry: { type: 'LineString', coordinates: coordinates ?? [] },
      properties: {},
    }
    map.getSource<GeoJSONSource>(SOURCES.route)?.setData({
      type: 'FeatureCollection',
      features: coordinates?.length ? [feature] : [],
    })
  }, [map, coordinates])
}

export default useRouteLayer
