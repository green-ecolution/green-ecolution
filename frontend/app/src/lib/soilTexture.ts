import { SoilCondition } from '@green-ecolution/backend-client'

export interface SoilFractions {
  sand: number
  silt: number
  clay: number
}

export type SoilGroup = 'sand' | 'silt' | 'loam' | 'clay'

export interface SoilRegion {
  condition: SoilCondition
  clay: [number, number]
  silt: [number, number]
  group: SoilGroup
}

// KA5 Bodenartendiagramm: each fine soil class is an axis-aligned rectangle
// in (silt, clay) space, clipped by silt + clay <= 100. Array order matters:
// it breaks ties deterministically for corner points on the hypotenuse.
export const SOIL_REGIONS: SoilRegion[] = [
  { condition: SoilCondition.Ss, clay: [0, 5], silt: [0, 10], group: 'sand' },
  { condition: SoilCondition.Su2, clay: [0, 5], silt: [10, 25], group: 'sand' },
  { condition: SoilCondition.Su3, clay: [0, 8], silt: [25, 40], group: 'sand' },
  { condition: SoilCondition.Su4, clay: [0, 8], silt: [40, 50], group: 'sand' },
  { condition: SoilCondition.Sl2, clay: [5, 8], silt: [10, 25], group: 'sand' },
  { condition: SoilCondition.Sl3, clay: [8, 12], silt: [10, 40], group: 'sand' },
  { condition: SoilCondition.Sl4, clay: [12, 17], silt: [10, 40], group: 'sand' },
  { condition: SoilCondition.Slu, clay: [8, 17], silt: [40, 50], group: 'sand' },
  { condition: SoilCondition.St2, clay: [5, 17], silt: [0, 10], group: 'sand' },
  { condition: SoilCondition.St3, clay: [17, 25], silt: [0, 15], group: 'sand' },
  { condition: SoilCondition.Us, clay: [0, 8], silt: [50, 80], group: 'silt' },
  { condition: SoilCondition.Uls, clay: [8, 17], silt: [50, 65], group: 'silt' },
  { condition: SoilCondition.Ut2, clay: [8, 12], silt: [65, 92], group: 'silt' },
  { condition: SoilCondition.Uu, clay: [0, 8], silt: [80, 100], group: 'silt' },
  { condition: SoilCondition.Ut3, clay: [12, 17], silt: [65, 88], group: 'silt' },
  { condition: SoilCondition.Ut4, clay: [17, 25], silt: [65, 83], group: 'silt' },
  { condition: SoilCondition.Ls2, clay: [17, 25], silt: [40, 50], group: 'loam' },
  { condition: SoilCondition.Ls3, clay: [17, 25], silt: [30, 40], group: 'loam' },
  { condition: SoilCondition.Ls4, clay: [17, 25], silt: [15, 30], group: 'loam' },
  { condition: SoilCondition.Lu, clay: [17, 30], silt: [50, 65], group: 'loam' },
  { condition: SoilCondition.Lt2, clay: [25, 35], silt: [30, 50], group: 'loam' },
  { condition: SoilCondition.Lt3, clay: [35, 45], silt: [30, 50], group: 'loam' },
  { condition: SoilCondition.Lts, clay: [25, 45], silt: [15, 30], group: 'loam' },
  { condition: SoilCondition.Ts4, clay: [25, 35], silt: [0, 15], group: 'clay' },
  { condition: SoilCondition.Ts3, clay: [35, 45], silt: [0, 15], group: 'clay' },
  { condition: SoilCondition.Ts2, clay: [45, 65], silt: [0, 15], group: 'clay' },
  { condition: SoilCondition.Tl, clay: [45, 65], silt: [15, 30], group: 'clay' },
  { condition: SoilCondition.Tu2, clay: [45, 65], silt: [30, 55], group: 'clay' },
  { condition: SoilCondition.Tu3, clay: [30, 45], silt: [50, 65], group: 'clay' },
  { condition: SoilCondition.Tu4, clay: [25, 35], silt: [65, 75], group: 'clay' },
  { condition: SoilCondition.Tt, clay: [65, 100], silt: [0, 35], group: 'clay' },
]

const matches = (region: SoilRegion, silt: number, clay: number, inclusiveUpper: boolean) => {
  const within = (value: number, [min, max]: [number, number]) =>
    value >= min && (value < max || max === 100 || (inclusiveUpper && value <= max))
  return within(clay, region.clay) && within(silt, region.silt)
}

export function classifySoilTexture(silt: number, clay: number): SoilCondition {
  if (silt < 0 || clay < 0 || silt + clay > 100) {
    throw new RangeError(`soil fractions outside the texture triangle: silt=${silt}, clay=${clay}`)
  }
  // Strict pass: lower bounds inclusive, upper bounds exclusive (upper 100 inclusive).
  // Only hypotenuse points can fall through; the inclusive pass catches those.
  const region =
    SOIL_REGIONS.find((r) => matches(r, silt, clay, false)) ??
    SOIL_REGIONS.find((r) => matches(r, silt, clay, true))
  if (!region) {
    throw new RangeError(`no KA5 region for silt=${silt}, clay=${clay}`)
  }
  return region.condition
}

const FRACTION_ORDER: (keyof SoilFractions)[] = ['sand', 'silt', 'clay']

export function balanceFractions(
  current: SoilFractions,
  changed: keyof SoilFractions,
  value: number,
): SoilFractions {
  const clamped = Math.min(100, Math.max(0, Math.round(Number.isFinite(value) ? value : 0)))
  const rest = 100 - clamped
  const [first, second] = FRACTION_ORDER.filter((field) => field !== changed)
  const oldSum = current[first] + current[second]
  const firstShare =
    oldSum > 0 ? Math.floor((rest * current[first]) / oldSum) : Math.floor(rest / 2)
  const next: SoilFractions = { ...current }
  next[changed] = clamped
  next[first] = firstShare
  next[second] = rest - firstShare
  return next
}

export function regionMidpoint(condition: SoilCondition): SoilFractions | null {
  const region = SOIL_REGIONS.find((r) => r.condition === condition)
  if (!region) return null
  const clay = Math.round((region.clay[0] + region.clay[1]) / 2)
  const silt = Math.min(Math.round((region.silt[0] + region.silt[1]) / 2), 100 - clay)
  return { sand: 100 - silt - clay, silt, clay }
}
