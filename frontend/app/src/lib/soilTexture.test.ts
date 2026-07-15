import { describe, it, expect } from 'vitest'
import { SoilCondition } from '@green-ecolution/backend-client'
import { SOIL_REGIONS, classifySoilTexture } from './soilTexture'

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
