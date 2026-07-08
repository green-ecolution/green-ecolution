import type { WateringStatus } from '@green-ecolution/backend-client'
import type { ExpressionSpecification } from 'maplibre-gl'

export const OPENFREEMAP_STYLE_URL = 'https://tiles.openfreemap.org/styles/liberty'

export const STATUS_COLORS: Record<WateringStatus, string> = {
  good: '#ACB63B',
  moderate: '#FFC434',
  bad: '#E44E4D',
  'just watered': '#747474',
  unknown: '#A2A2A2',
}

// Reads the `status` feature property (a WateringStatus wire value) → colorHex.
export const STATUS_COLOR_EXPRESSION: ExpressionSpecification = [
  'match',
  ['get', 'status'],
  'good',
  STATUS_COLORS.good,
  'moderate',
  STATUS_COLORS.moderate,
  'bad',
  STATUS_COLORS.bad,
  'just watered',
  STATUS_COLORS['just watered'],
  STATUS_COLORS.unknown,
]

// Below this zoom the map shows one marker per watering group; at/above it the
// groups give way to individual tree points.
export const TREE_ZOOM_THRESHOLD = 17

export const SOURCES = {
  treePoints: 'gec-tree-points',
  clusterMarkers: 'gec-cluster-markers',
  clusterBoundaries: 'gec-cluster-boundaries',
  selectTrees: 'gec-select-trees',
  selectClusters: 'gec-select-clusters',
  route: 'gec-route',
} as const

export const LAYERS = {
  treePoints: 'gec-tree-points',
  treeIcon: 'gec-tree-icon',
  clusterMarkers: 'gec-cluster-markers',
  clusterMarkerCount: 'gec-cluster-marker-count',
  boundaryFill: 'gec-boundary-fill',
  boundaryLine: 'gec-boundary-line',
  selectTreePoints: 'gec-select-tree-points',
  selectTreeIcon: 'gec-select-tree-icon',
  selectTreeCheck: 'gec-select-tree-check',
  selectClusterPoints: 'gec-select-cluster-points',
  selectClusterCount: 'gec-select-cluster-count',
  routeCasing: 'gec-route-casing',
  routeLine: 'gec-route-line',
} as const

export const ROUTE_COLORS = {
  // hex approximation of the UI theme's --green-dark (MapLibre can't parse oklch)
  line: '#4B6E3F',
  casing: '#ffffff',
} as const

export const TREE_ICON_IMAGE = 'gec-tree-icon-image'

// White two-leaf tree glyph (from components/icons/Tree) rasterised for the
// symbol layer; the colored circle underneath carries the watering status.
const TREE_ICON_SVG = `<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none"><path d="M3.5607 3.69226C10.4781 3.69246 12 6.66426 12 10.3114C12 6.66426 13.5219 3.69246 20.4393 3.69226C20.4017 7.31032 18.8328 11.5272 12 11.5272C5.16719 11.5272 3.59831 7.31032 3.5607 3.69226Z" stroke="#ffffff" stroke-width="2" stroke-linejoin="round"/><path d="M3.5607 12.6079C10.4781 12.6081 12 15.5799 12 19.2271C12 15.5799 13.5219 12.6081 20.4393 12.6079C20.4017 16.2259 18.8328 20.4428 12 20.4428C5.16719 20.4428 3.59831 16.2259 3.5607 12.6079Z" stroke="#ffffff" stroke-width="2" stroke-linejoin="round"/></svg>`

export const TREE_ICON_URL = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(TREE_ICON_SVG)}`

export const CHECK_ICON_IMAGE = 'gec-check-icon-image'

// White check glyph shown on selected trees during selection.
const CHECK_ICON_SVG = `<svg xmlns="http://www.w3.org/2000/svg" width="48" height="48" viewBox="0 0 24 24" fill="none"><path d="M20 6 9 17l-5-5" stroke="#ffffff" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/></svg>`

export const CHECK_ICON_URL = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(CHECK_ICON_SVG)}`
