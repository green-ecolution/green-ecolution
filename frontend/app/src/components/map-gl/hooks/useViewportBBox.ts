import { useEffect, useRef, useState } from 'react'
import type { Map as MaplibreMap } from 'maplibre-gl'
import type { BoundingBox } from '@/api/queries'
import { useMaplibreMap } from '../MapContext'

const BUFFER = 1.5
const DEBOUNCE_MS = 200

const expand = (b: BoundingBox, factor: number): BoundingBox => {
  const latPad = ((b.neLat - b.swLat) * (factor - 1)) / 2
  const lngPad = ((b.neLng - b.swLng) * (factor - 1)) / 2
  return {
    swLat: b.swLat - latPad,
    swLng: b.swLng - lngPad,
    neLat: b.neLat + latPad,
    neLng: b.neLng + lngPad,
  }
}

const fitsInside = (inner: BoundingBox, outer: BoundingBox): boolean =>
  inner.swLat >= outer.swLat &&
  inner.swLng >= outer.swLng &&
  inner.neLat <= outer.neLat &&
  inner.neLng <= outer.neLng

const readBounds = (map: MaplibreMap): BoundingBox => {
  const b = map.getBounds()
  return { swLat: b.getSouth(), swLng: b.getWest(), neLat: b.getNorth(), neLng: b.getEast() }
}

// Buffered (1.5x) viewport bbox; only changes when the view pans outside the
// previously-fetched buffer, so the tree query refetches rarely.
const useViewportBBox = (): BoundingBox => {
  const map = useMaplibreMap()
  const [bbox, setBbox] = useState<BoundingBox>(() => expand(readBounds(map), BUFFER))
  const bboxRef = useRef(bbox)
  useEffect(() => {
    bboxRef.current = bbox
  }, [bbox])

  useEffect(() => {
    let timer: ReturnType<typeof setTimeout> | null = null
    const recompute = () => {
      const current = readBounds(map)
      const prev = bboxRef.current
      if (fitsInside(current, prev)) return
      const next = expand(current, BUFFER)
      bboxRef.current = next
      setBbox(next)
    }
    const onMove = () => {
      if (timer) clearTimeout(timer)
      timer = setTimeout(recompute, DEBOUNCE_MS)
    }
    map.on('moveend', onMove)
    map.on('zoomend', onMove)
    return () => {
      if (timer) clearTimeout(timer)
      map.off('moveend', onMove)
      map.off('zoomend', onMove)
    }
  }, [map])

  return bbox
}

export default useViewportBBox
