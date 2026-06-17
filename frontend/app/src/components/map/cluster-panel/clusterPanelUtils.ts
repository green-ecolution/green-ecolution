import type { TreeResponse } from '@/api/backendApi'

export const summarizeTopSpecies = (trees: TreeResponse[], limit = 2): string => {
  const counts = new Map<string, number>()
  for (const { species } of trees) {
    counts.set(species, (counts.get(species) ?? 0) + 1)
  }
  return [...counts.entries()]
    .sort((a, b) => b[1] - a[1])
    .slice(0, limit)
    .map(([species]) => species)
    .join(', ')
}
