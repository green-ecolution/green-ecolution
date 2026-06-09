import type { ClusterMarkerResponse, TreeResponse } from '@/api/backendApi'

export const sortTreesSensorFirst = (trees: TreeResponse[]): TreeResponse[] =>
  trees
    .map((tree, index) => ({ tree, index }))
    .sort((a, b) => {
      const aHasSensor = a.tree.sensor != null ? 0 : 1
      const bHasSensor = b.tree.sensor != null ? 0 : 1
      return aHasSensor - bHasSensor || a.index - b.index
    })
    .map(({ tree }) => tree)

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

export const filterMarkersByName = (
  markers: ClusterMarkerResponse[],
  term: string,
): ClusterMarkerResponse[] => {
  const needle = term.trim().toLowerCase()
  if (!needle) return markers
  return markers.filter((m) => m.name.toLowerCase().includes(needle))
}
