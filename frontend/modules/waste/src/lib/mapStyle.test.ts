import { describe, expect, it } from 'vitest'
import { wasteBinFixtures } from '../test/handlers'
import { binsToGeoJson } from './mapStyle'

describe('binsToGeoJson', () => {
  it('orders coordinates as [lon, lat] and keeps id, number and fill state', () => {
    const [bin] = wasteBinFixtures
    const { type, features } = binsToGeoJson([bin])

    expect(type).toBe('FeatureCollection')
    expect(features).toHaveLength(1)
    expect(features[0].geometry).toEqual({
      type: 'Point',
      coordinates: [bin.longitude, bin.latitude],
    })
    expect(features[0].properties).toEqual({
      id: bin.id,
      number: bin.number,
      fillState: bin.fillState,
    })
  })

  it('returns an empty collection for no bins', () => {
    expect(binsToGeoJson([]).features).toEqual([])
  })
})
