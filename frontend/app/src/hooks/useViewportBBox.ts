import { useEffect, useRef, useState } from 'react'
import { useMap } from 'react-leaflet'
import type { BoundingBox } from '@/api/queries'

const DEFAULT_BUFFER = 1.5
const DEFAULT_DEBOUNCE_MS = 200

export const expandBBox = (b: BoundingBox, factor: number): BoundingBox => {
  const latPad = ((b.neLat - b.swLat) * (factor - 1)) / 2
  const lngPad = ((b.neLng - b.swLng) * (factor - 1)) / 2
  return {
    swLat: b.swLat - latPad,
    swLng: b.swLng - lngPad,
    neLat: b.neLat + latPad,
    neLng: b.neLng + lngPad,
  }
}

export const fitsInside = (inner: BoundingBox, outer: BoundingBox): boolean =>
  inner.swLat >= outer.swLat &&
  inner.swLng >= outer.swLng &&
  inner.neLat <= outer.neLat &&
  inner.neLng <= outer.neLng

export interface UseViewportBBoxOptions {
  bufferFactor?: number
  debounceMs?: number
}

export function useViewportBBox(opts?: UseViewportBBoxOptions): BoundingBox | null {
  const map = useMap()
  const buffer = opts?.bufferFactor ?? DEFAULT_BUFFER
  const debounceMs = opts?.debounceMs ?? DEFAULT_DEBOUNCE_MS

  const [bbox, setBBox] = useState<BoundingBox | null>(() => {
    if (!map) return null
    const b = map.getBounds()
    return expandBBox(
      {
        swLat: b.getSouth(),
        swLng: b.getWest(),
        neLat: b.getNorth(),
        neLng: b.getEast(),
      },
      buffer,
    )
  })

  // Ref tracks the current buffered bbox synchronously so the debounced
  // comparison sees the latest value without waiting for a render cycle.
  const bboxRef = useRef(bbox)
  bboxRef.current = bbox

  useEffect(() => {
    if (!map) return
    let timer: ReturnType<typeof setTimeout> | null = null

    const recompute = () => {
      const b = map.getBounds()
      const current: BoundingBox = {
        swLat: b.getSouth(),
        swLng: b.getWest(),
        neLat: b.getNorth(),
        neLng: b.getEast(),
      }
      const prev = bboxRef.current
      if (prev && fitsInside(current, prev)) return
      const next = expandBBox(current, buffer)
      bboxRef.current = next
      setBBox(next)
    }

    const onMove = () => {
      if (timer) clearTimeout(timer)
      timer = setTimeout(recompute, debounceMs)
    }

    map.on('moveend', onMove)
    map.on('zoomend', onMove)

    if (!bboxRef.current) recompute()

    return () => {
      if (timer) clearTimeout(timer)
      map.off('moveend', onMove)
      map.off('zoomend', onMove)
    }
  }, [map, buffer, debounceMs])

  return bbox
}
