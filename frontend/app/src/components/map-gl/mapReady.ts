import type { Map as MaplibreMap } from 'maplibre-gl'

// maplibre clears its internal `style` on remove(); calling getLayer/getSource/
// removeLayer/etc. afterwards throws "this.style is undefined". This guards
// operations that can run while the map is being torn down (effect cleanups or
// late callbacks racing a route change).
export const isMapAlive = (map: MaplibreMap): boolean =>
  Boolean((map as unknown as { style?: unknown }).style)
