import type { FeatureCollection, Point } from 'geojson'
import type { ExpressionSpecification } from 'maplibre-gl'
import type { FillStateDto, WasteBinResponse } from '@green-ecolution/backend-client'

export const SOURCES = { bins: 'waste-bins' } as const
export const LAYERS = { bins: 'waste-bins', selected: 'waste-bins-selected' } as const

// A click on this layer opens the popup, so it does not count as a background click.
export const INTERACTIVE_LAYERS = [LAYERS.bins] as const

// Same palette as the irrigation status colours: red = act, green = fine, greys = nothing to do.
export const FILL_STATE_COLORS: Record<FillStateDto, string> = {
  full: '#E44E4D',
  normal: '#ACB63B',
  empty: '#747474',
  unknown: '#A2A2A2',
}

export const FILL_STATE_COLOR_EXPRESSION: ExpressionSpecification = [
  'match',
  ['get', 'fillState'],
  'full',
  FILL_STATE_COLORS.full,
  'normal',
  FILL_STATE_COLORS.normal,
  'empty',
  FILL_STATE_COLORS.empty,
  FILL_STATE_COLORS.unknown,
]

export interface BinFeatureProperties {
  id: string
  fillState: string
  number: string
}

export const binsToGeoJson = (
  bins: WasteBinResponse[],
): FeatureCollection<Point, BinFeatureProperties> => ({
  type: 'FeatureCollection',
  features: bins.map((bin) => ({
    type: 'Feature',
    geometry: { type: 'Point', coordinates: [bin.longitude, bin.latitude] },
    properties: { id: bin.id, fillState: bin.fillState, number: bin.number },
  })),
})
