import { describe, it, expect } from 'vitest'
import { SoilCondition } from '@green-ecolution/backend-client'
import {
  SOIL_REGIONS,
  classifySoilTexture,
  balanceFractions,
  regionMidpoint,
  regionPolygon,
  polygonCentroid,
} from './soilTexture'

// One interior point per KA5 class: [silt, clay, expected]
const INTERIOR_POINTS: [number, number, SoilCondition][] = [
  [5, 2, SoilCondition.Ss],
  [17, 2, SoilCondition.Su2],
  [30, 4, SoilCondition.Su3],
  [45, 4, SoilCondition.Su4],
  [17, 6, SoilCondition.Sl2],
  [25, 10, SoilCondition.Sl3],
  [25, 14, SoilCondition.Sl4],
  [45, 12, SoilCondition.Slu],
  [5, 10, SoilCondition.St2],
  [7, 20, SoilCondition.St3],
  [65, 4, SoilCondition.Us],
  [90, 4, SoilCondition.Uu],
  [57, 12, SoilCondition.Uls],
  [75, 10, SoilCondition.Ut2],
  [75, 14, SoilCondition.Ut3],
  [70, 20, SoilCondition.Ut4],
  [45, 20, SoilCondition.Ls2],
  [35, 20, SoilCondition.Ls3],
  [22, 20, SoilCondition.Ls4],
  [57, 25, SoilCondition.Lu],
  [40, 30, SoilCondition.Lt2],
  [40, 40, SoilCondition.Lt3],
  [22, 35, SoilCondition.Lts],
  [7, 30, SoilCondition.Ts4],
  [7, 40, SoilCondition.Ts3],
  [7, 55, SoilCondition.Ts2],
  [22, 55, SoilCondition.Tl],
  [40, 55, SoilCondition.Tu2],
  [57, 40, SoilCondition.Tu3],
  [70, 28, SoilCondition.Tu4],
  [17, 80, SoilCondition.Tt],
]

describe('classifySoilTexture', () => {
  it.each(INTERIOR_POINTS)('classifies silt=%i clay=%i as %s', (silt, clay, expected) => {
    expect(classifySoilTexture(silt, clay)).toBe(expected)
  })

  it('treats lower bounds as inclusive and upper bounds as exclusive', () => {
    expect(classifySoilTexture(10, 5)).toBe(SoilCondition.Sl2)
    expect(classifySoilTexture(25, 0)).toBe(SoilCondition.Su3)
    expect(classifySoilTexture(0, 17)).toBe(SoilCondition.St3)
  })

  it('classifies points on the hypotenuse', () => {
    expect(classifySoilTexture(92, 8)).toBe(SoilCondition.Ut2)
    expect(classifySoilTexture(100, 0)).toBe(SoilCondition.Uu)
    expect(classifySoilTexture(0, 100)).toBe(SoilCondition.Tt)
  })

  it('throws on inputs outside the triangle', () => {
    expect(() => classifySoilTexture(60, 50)).toThrow(RangeError)
    expect(() => classifySoilTexture(-1, 0)).toThrow(RangeError)
    expect(() => classifySoilTexture(0, -1)).toThrow(RangeError)
  })

  it('covers every integer point in the triangle without overlaps', () => {
    const strictMatches = (silt: number, clay: number) =>
      SOIL_REGIONS.filter(
        (r) =>
          clay >= r.clay[0] &&
          (clay < r.clay[1] || r.clay[1] === 100) &&
          silt >= r.silt[0] &&
          (silt < r.silt[1] || r.silt[1] === 100),
      )

    for (let silt = 0; silt <= 100; silt++) {
      for (let clay = 0; clay + silt <= 100; clay++) {
        const matches = strictMatches(silt, clay)
        expect(matches.length, `silt=${silt} clay=${clay}`).toBeLessThanOrEqual(1)
        if (matches.length === 0) {
          // Only hypotenuse points may fall through to the inclusive pass.
          expect(silt + clay, `gap at silt=${silt} clay=${clay}`).toBe(100)
        }
        expect(() => classifySoilTexture(silt, clay)).not.toThrow()
      }
    }
  })
})

describe('balanceFractions', () => {
  it('keeps the changed field and distributes the rest proportionally', () => {
    expect(balanceFractions({ sand: 65, silt: 25, clay: 10 }, 'clay', 80)).toEqual({
      sand: 14,
      silt: 6,
      clay: 80,
    })
  })

  it('splits evenly when the other two fields are zero', () => {
    expect(balanceFractions({ sand: 0, silt: 0, clay: 100 }, 'clay', 40)).toEqual({
      sand: 30,
      silt: 30,
      clay: 40,
    })
  })

  it('clamps the changed value to 0..100 and rounds to integers', () => {
    expect(balanceFractions({ sand: 33, silt: 34, clay: 33 }, 'sand', 150)).toEqual({
      sand: 100,
      silt: 0,
      clay: 0,
    })
    expect(balanceFractions({ sand: 33, silt: 34, clay: 33 }, 'sand', -5).sand).toBe(0)
    expect(balanceFractions({ sand: 33, silt: 34, clay: 33 }, 'sand', 40.6).sand).toBe(41)
  })

  it('treats NaN (empty input) as 0', () => {
    expect(balanceFractions({ sand: 33, silt: 34, clay: 33 }, 'silt', Number.NaN).silt).toBe(0)
  })

  it('always sums to exactly 100', () => {
    for (let value = 0; value <= 100; value++) {
      const result = balanceFractions({ sand: 61, silt: 22, clay: 17 }, 'silt', value)
      expect(result.sand + result.silt + result.clay).toBe(100)
    }
  })
})

describe('regionMidpoint', () => {
  it('returns the integer midpoint of the region', () => {
    expect(regionMidpoint(SoilCondition.Sl3)).toEqual({ sand: 65, silt: 25, clay: 10 })
  })

  it('clamps hypotenuse-clipped regions into the triangle', () => {
    expect(regionMidpoint(SoilCondition.Tt)).toEqual({ sand: 0, silt: 17, clay: 83 })
  })

  it('returns null for conditions without a triangle region', () => {
    expect(regionMidpoint(SoilCondition.FS)).toBeNull()
    expect(regionMidpoint(SoilCondition.MS)).toBeNull()
    expect(regionMidpoint(SoilCondition.GS)).toBeNull()
    expect(regionMidpoint(SoilCondition.Unknown)).toBeNull()
  })

  it('classifies every region midpoint back to its own condition', () => {
    for (const region of SOIL_REGIONS) {
      const midpoint = regionMidpoint(region.condition)
      expect(midpoint).not.toBeNull()
      const { sand, silt, clay } = midpoint!
      expect(sand).toBeGreaterThanOrEqual(0)
      expect(sand + silt + clay).toBe(100)
      expect(classifySoilTexture(silt, clay)).toBe(region.condition)
    }
  })
})

describe('regionPolygon / polygonCentroid', () => {
  it('returns the plain rectangle for unclipped regions', () => {
    const sl3 = SOIL_REGIONS.find((r) => r.condition === SoilCondition.Sl3)!
    expect(regionPolygon(sl3)).toEqual([
      { silt: 10, clay: 8 },
      { silt: 40, clay: 8 },
      { silt: 40, clay: 12 },
      { silt: 10, clay: 12 },
    ])
  })

  it('clips regions at the hypotenuse', () => {
    const tt = SOIL_REGIONS.find((r) => r.condition === SoilCondition.Tt)!
    expect(regionPolygon(tt)).toEqual([
      { silt: 0, clay: 65 },
      { silt: 35, clay: 65 },
      { silt: 0, clay: 100 },
    ])
  })

  it('keeps every region polygon inside the triangle', () => {
    for (const region of SOIL_REGIONS) {
      const polygon = regionPolygon(region)
      expect(polygon.length).toBeGreaterThanOrEqual(3)
      for (const point of polygon) {
        expect(point.silt + point.clay).toBeLessThanOrEqual(100.000001)
      }
      polygon.forEach((point, i) => {
        const prev = polygon[(i + polygon.length - 1) % polygon.length]
        expect(point.silt !== prev.silt || point.clay !== prev.clay).toBe(true)
      })
    }
  })

  it('computes the centroid of a square', () => {
    const centroid = polygonCentroid([
      { silt: 0, clay: 0 },
      { silt: 10, clay: 0 },
      { silt: 10, clay: 10 },
      { silt: 0, clay: 10 },
    ])
    expect(centroid.silt).toBeCloseTo(5)
    expect(centroid.clay).toBeCloseTo(5)
  })
})
