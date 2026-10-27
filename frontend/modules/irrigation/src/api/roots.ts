import type { QueryKey } from '@tanstack/react-query'

/**
 * Every key prefix the irrigation aggregates own. The roots grew inconsistent
 * ('treecluster' vs 'treeclusters' vs 'clusters'), so invalidate through this
 * table instead of guessing a prefix. Kept apart from the queries so the
 * manifest can name them without loading the API client.
 */
export const irrigationRoots = {
  cluster: [['treecluster'], ['treeclusters'], ['clusters']],
  tree: [['tree'], ['trees'], ['planting-years']],
  evaluation: [['evaluation']],
} as const satisfies Record<string, readonly QueryKey[]>
