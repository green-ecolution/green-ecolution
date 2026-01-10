import L from 'leaflet'
import { useCallback, useEffect, useRef } from 'react'
import { useMap } from 'react-leaflet'

interface WithLocation {
  latitude: number
  longitude: number
}

interface MarkerListProps<T extends WithLocation> {
  data: T[]
  onClick?: (data: T) => void
  icon: ((data: T) => L.DivIcon) | L.DivIcon
  tooltipContent?: ((data: T, layer: L.Layer) => L.Content) | L.Tooltip | L.Content
  tooltipOptions?: L.TooltipOptions
  getId?: (data: T) => string | number
}

const getDefaultId = <T extends WithLocation>(item: T): string =>
  `${item.latitude}-${item.longitude}`

const MarkerList = <T extends WithLocation>({
  data,
  onClick,
  icon,
  tooltipContent,
  tooltipOptions,
  getId = getDefaultId,
}: MarkerListProps<T>) => {
  const map = useMap()
  const markerMapRef = useRef<Map<string | number, { marker: L.Marker; data: T }>>(new Map())
  const dataRef = useRef<T[]>(data)
  const iconRef = useRef(icon)
  const onClickRef = useRef(onClick)

  useEffect(() => {
    onClickRef.current = onClick
    iconRef.current = icon
  })

  const updateVisibleMarkers = useCallback(() => {
    if (!map) return

    const bounds = map.getBounds()
    const currentData = dataRef.current
    const markerMap = markerMapRef.current

    markerMap.forEach(({ marker }) => {
      const pos = marker.getLatLng()
      if (bounds.contains(pos)) {
        if (!map.hasLayer(marker)) {
          marker.addTo(map)
        }
      } else {
        if (map.hasLayer(marker)) {
          map.removeLayer(marker)
        }
      }
    })

    currentData.forEach((item) => {
      const id = getId(item)
      if (!markerMap.has(id) && bounds.contains([item.latitude, item.longitude])) {
        const markerIcon =
          typeof iconRef.current === 'function' ? iconRef.current(item) : iconRef.current
        const marker = L.marker([item.latitude, item.longitude], { icon: markerIcon })

        if (tooltipContent) {
          if (typeof tooltipContent === 'function') {
            marker.bindTooltip((m) => tooltipContent(item, m), tooltipOptions)
          } else {
            marker.bindTooltip(tooltipContent, tooltipOptions)
          }
        }

        marker.on('click', () => onClickRef.current?.(item))
        marker.addTo(map)
        markerMap.set(id, { marker, data: item })
      }
    })
  }, [map, getId, tooltipContent, tooltipOptions])

  useEffect(() => {
    dataRef.current = data
    const markerMap = markerMapRef.current
    const currentIds = new Set(data.map(getId))

    markerMap.forEach((entry, id) => {
      if (!currentIds.has(id)) {
        if (map) map.removeLayer(entry.marker)
        markerMap.delete(id)
      }
    })

    updateVisibleMarkers()
  }, [data, map, getId, updateVisibleMarkers])

  useEffect(() => {
    const markerMap = markerMapRef.current
    markerMap.forEach(({ marker, data: item }) => {
      const newIcon = typeof icon === 'function' ? icon(item) : icon
      marker.setIcon(newIcon)
    })
  }, [icon])

  useEffect(() => {
    const currentMarkerMap = markerMapRef.current
    let lastUpdate = 0
    const THROTTLE_MS = 150

    const throttledUpdate = () => {
      const now = Date.now()
      if (now - lastUpdate >= THROTTLE_MS) {
        lastUpdate = now
        updateVisibleMarkers()
      }
    }

    if (map) {
      map.on('move', throttledUpdate)
      map.on('moveend', updateVisibleMarkers)
    }

    return () => {
      if (map) {
        map.off('move', throttledUpdate)
        map.off('moveend', updateVisibleMarkers)
        currentMarkerMap.forEach(({ marker }) => map.removeLayer(marker))
        currentMarkerMap.clear()
      }
    }
  }, [map, updateVisibleMarkers])

  return null
}

export default MarkerList
