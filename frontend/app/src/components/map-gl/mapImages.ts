import type { Map as MaplibreMap } from 'maplibre-gl'

/** An image a symbol layer references by `id`; `url` is a 48 px square, drawn at pixel ratio 2. */
export interface MapImage {
  id: string
  url: string
}

/**
 * Adds every image the map does not have yet. `onReady` runs once per image as
 * soon as it is available, synchronously for one already registered. The
 * returned function cancels pending loads, so a late image never lands after
 * the caller has cleaned up.
 */
export function registerMapImages(
  map: MaplibreMap,
  images: readonly MapImage[],
  onReady: () => void,
): () => void {
  let cancelled = false
  for (const { id, url } of images) {
    if (map.hasImage(id)) {
      onReady()
      continue
    }
    const img = new Image(48, 48)
    img.onload = () => {
      if (cancelled) return
      if (!map.hasImage(id)) map.addImage(id, img, { pixelRatio: 2 })
      onReady()
    }
    img.src = url
  }
  return () => {
    cancelled = true
  }
}
