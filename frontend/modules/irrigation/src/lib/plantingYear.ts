/**
 * How far ahead a shorthand year is still read as a planned planting. Past the
 * window, `85` far more plausibly means an old tree than a planting in 2085.
 */
const PLANNING_WINDOW_YEARS = 20

/**
 * Resolves a shorthand planting year the way a person means it: `25` is 2025,
 * `29` a planting scheduled for 2029, `85` an old tree from 1985.
 *
 * Anything that is not a one- or two-digit whole number is passed through, so a
 * typo like `202` still reaches validation instead of being guessed at.
 */
export const expandShortYear = (
  value: number,
  referenceYear = new Date().getFullYear(),
): number => {
  if (!Number.isInteger(value) || value < 0 || value > 99) {
    return value
  }

  const century = Math.floor(referenceYear / 100) * 100
  const expanded = century + value

  return expanded <= referenceYear + PLANNING_WINDOW_YEARS ? expanded : expanded - 100
}
