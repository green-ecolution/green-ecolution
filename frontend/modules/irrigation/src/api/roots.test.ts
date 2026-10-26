import { describe, it, expect, vi } from 'vitest'
import { QueryClient } from '@tanstack/react-query'
import type { QueryKey } from '@tanstack/react-query'
import { invalidateRoots } from '@/lib/queryInvalidation'
import { irrigationPlanning } from '../planning'
import { clusterQueries, treeQueries } from './queries'
import { irrigationRoots } from './roots'

const CLUSTER_ID = '0190a8e9-7c4f-7000-8000-000000000001'
const TREE_ID = '0190a8e9-7c4f-7000-8000-000000000002'

const makeClient = () =>
  new QueryClient({ defaultOptions: { queries: { staleTime: 60_000, retry: false } } })

const seed = (queryClient: QueryClient, keys: QueryKey[]) =>
  keys.forEach((key) => queryClient.setQueryData(key, { seeded: true }))

const isInvalidated = (queryClient: QueryClient, key: QueryKey) =>
  queryClient.getQueryState(key)?.isInvalidated === true

const routerStub = () => ({ invalidate: vi.fn().mockResolvedValue(undefined) })

describe('irrigationRoots', () => {
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

    await invalidateRoots(queryClient, routerStub(), irrigationRoots.cluster)

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

    await invalidateRoots(queryClient, routerStub(), irrigationRoots.tree)

    keys.forEach((key) => expect(isInvalidated(queryClient, key)).toBe(true))
  })

  it('lets a plan status change reach clusters, trees and the evaluation', async () => {
    const queryClient = makeClient()
    const keys: QueryKey[] = [
      clusterQueries.list().queryKey,
      clusterQueries.stops([CLUSTER_ID]).queryKey,
      treeQueries.detail(TREE_ID).queryKey,
      ['evaluation'],
    ]
    seed(queryClient, keys)

    await invalidateRoots(queryClient, routerStub(), irrigationPlanning.invalidateOnStatusChange)

    keys.forEach((key) => expect(isInvalidated(queryClient, key)).toBe(true))
  })
})
