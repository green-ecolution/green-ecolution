import { describe, it, expect, vi } from 'vitest'
import { QueryClient } from '@tanstack/react-query'
import type { QueryKey } from '@tanstack/react-query'
import { clusterQueries, treeQueries, wateringPlanQueries } from '@/api/queries'
import { invalidateAggregates } from './queryInvalidation'

const CLUSTER_ID = '0190a8e9-7c4f-7000-8000-000000000001'
const TREE_ID = '0190a8e9-7c4f-7000-8000-000000000002'

const makeClient = () =>
  new QueryClient({ defaultOptions: { queries: { staleTime: 60_000, retry: false } } })

const seed = (queryClient: QueryClient, keys: QueryKey[]) =>
  keys.forEach((key) => queryClient.setQueryData(key, { seeded: true }))

const isInvalidated = (queryClient: QueryClient, key: QueryKey) =>
  queryClient.getQueryState(key)?.isInvalidated === true

const routerStub = () => ({ invalidate: vi.fn().mockResolvedValue(undefined) })

describe('invalidateAggregates', () => {
  it('reaches every key root a cluster query can live under', async () => {
    const queryClient = makeClient()
    const keys: QueryKey[] = [
      clusterQueries.detail(CLUSTER_ID).queryKey,
      clusterQueries.list().queryKey,
      clusterQueries.statistics().queryKey,
      clusterQueries.markers().queryKey,
      clusterQueries.boundaries().queryKey,
      clusterQueries.soilMoisture(CLUSTER_ID, { bucket: 'day' }).queryKey,
    ]
    seed(queryClient, keys)

    await invalidateAggregates(queryClient, routerStub(), ['cluster'])

    keys.forEach((key) => expect(isInvalidated(queryClient, key)).toBe(true))
  })

  it('reaches every key root a tree query can live under', async () => {
    const queryClient = makeClient()
    const keys: QueryKey[] = [
      treeQueries.detail(TREE_ID).queryKey,
      treeQueries.list().queryKey,
      treeQueries.plantingYears().queryKey,
      treeQueries.nearest({ lat: 54.78, lng: 9.44 }).queryKey,
      treeQueries.markers({ bbox: { swLat: 54.7, swLng: 9.4, neLat: 54.8, neLng: 9.5 } }).queryKey,
    ]
    seed(queryClient, keys)

    await invalidateAggregates(queryClient, routerStub(), ['tree'])

    keys.forEach((key) => expect(isInvalidated(queryClient, key)).toBe(true))
  })

  it('leaves unrelated aggregates alone', async () => {
    const queryClient = makeClient()
    seed(queryClient, [
      clusterQueries.detail(CLUSTER_ID).queryKey,
      treeQueries.detail(TREE_ID).queryKey,
    ])

    await invalidateAggregates(queryClient, routerStub(), ['wateringPlan'])

    expect(isInvalidated(queryClient, clusterQueries.detail(CLUSTER_ID).queryKey)).toBe(false)
    expect(isInvalidated(queryClient, treeQueries.detail(TREE_ID).queryKey)).toBe(false)
  })

  it('invalidates several aggregates at once', async () => {
    const queryClient = makeClient()
    const planKey = wateringPlanQueries.detail(CLUSTER_ID).queryKey
    seed(queryClient, [
      planKey,
      clusterQueries.list().queryKey,
      treeQueries.detail(TREE_ID).queryKey,
    ])

    await invalidateAggregates(queryClient, routerStub(), ['wateringPlan', 'cluster', 'tree'])

    expect(isInvalidated(queryClient, planKey)).toBe(true)
    expect(isInvalidated(queryClient, clusterQueries.list().queryKey)).toBe(true)
    expect(isInvalidated(queryClient, treeQueries.detail(TREE_ID).queryKey)).toBe(true)
  })

  it('only re-runs the route loaders when asked to', async () => {
    const queryClient = makeClient()

    const quiet = routerStub()
    await invalidateAggregates(queryClient, quiet, ['cluster'])
    expect(quiet.invalidate).not.toHaveBeenCalled()

    const reloading = routerStub()
    await invalidateAggregates(queryClient, reloading, ['cluster'], { reloadRoutes: true })
    expect(reloading.invalidate).toHaveBeenCalledTimes(1)
  })
})
