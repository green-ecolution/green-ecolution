import { useCallback } from 'react'
import { useQueryClient, type QueryClient, type QueryKey } from '@tanstack/react-query'
import { useRouter, type RegisteredRouter } from '@tanstack/react-router'
import { queryRoots } from '@/api/queries'
import { irrigationRoots } from '../../../modules/irrigation/src/api/queries'

/** The only router capability this module needs. */
type RouteReloader = Pick<RegisteredRouter, 'invalidate'>

export interface InvalidateOptions {
  /**
   * Also re-run the loaders of the active routes. Needed when the mutation
   * keeps the user on an `entityRoute` page: those read the entity from loader
   * data, which no query subscription reaches. Leave it off when the caller
   * navigates afterwards — the navigation re-runs the loaders anyway — and
   * especially after a delete, where re-running the loader of the now-missing
   * entity renders EntityNotFound before the redirect lands.
   */
  reloadRoutes?: boolean
}

/**
 * Invalidate every given query root (`queryRoots.x`, or a module's own roots).
 *
 * Callers that navigate right after do not need to await this:
 * `invalidateQueries` marks the entries stale synchronously, so a route loader
 * running in the same tick already refetches.
 */
export const invalidateRoots = async (
  queryClient: QueryClient,
  router: RouteReloader,
  roots: readonly QueryKey[],
  { reloadRoutes = false }: InvalidateOptions = {},
): Promise<void> => {
  await Promise.all(roots.map((queryKey) => queryClient.invalidateQueries({ queryKey })))
  if (reloadRoutes) await router.invalidate()
}

/**
 * Query roots touched by a watering plan status change: finishing a plan flags
 * its clusters and their trees as just watered, and the evaluation dashboard
 * aggregates finished plans.
 */
export const PLAN_STATUS_AGGREGATES: readonly QueryKey[] = [
  ...queryRoots.wateringPlan,
  ...irrigationRoots.cluster,
  ...irrigationRoots.tree,
  ...irrigationRoots.evaluation,
]

export const useInvalidateAggregates = () => {
  const queryClient = useQueryClient()
  const router = useRouter()

  return useCallback(
    (roots: readonly QueryKey[], options?: InvalidateOptions) =>
      invalidateRoots(queryClient, router, roots, options),
    [queryClient, router],
  )
}
