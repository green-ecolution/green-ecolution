export const OPENFREEMAP_STYLE_URL = 'https://tiles.openfreemap.org/styles/liberty'

export const SOURCES = {
  route: 'gec-route',
} as const

export const LAYERS = {
  routeCasing: 'gec-route-casing',
  routeLine: 'gec-route-line',
} as const

export const ROUTE_COLORS = {
  // hex approximation of the UI theme's --green-dark (MapLibre can't parse oklch)
  line: '#4B6E3F',
  casing: '#ffffff',
} as const
