import { describe, it, expect, vi } from 'vitest'
import { QueryClient } from '@tanstack/react-query'
import type { QueryKey } from '@tanstack/react-query'
import { queryRoots, vehicleQueries, wateringPlanQueries } from '@/api/queries'
import { invalidateRoots, planStatusQueryRoots } from './queryInvalidation'

const PLAN_ID = '0190a8e9-7c4f-7000-8000-000000000001'
const VEHICLE_ID = '0190a8e9-7c4f-7000-8000-000000000002'
const MODULE_KEY: QueryKey = ['module-thing', 'x']

const makeClient = () =>
  new QueryClient({ defaultOptions: { queries: { staleTime: 60_000, retry: false } } })

const seed = (queryClient: QueryClient, keys: QueryKey[]) =>
  keys.forEach((key) => queryClient.setQueryData(key, { seeded: true }))

const isInvalidated = (queryClient: QueryClient, key: QueryKey) =>
  queryClient.getQueryState(key)?.isInvalidated === true

const routerStub = () => ({ invalidate: vi.fn().mockResolvedValue(undefined) })

describe('invalidateRoots', () => {
  it('leaves unrelated aggregates alone', async () => {
    const queryClient = makeClient()
    seed(queryClient, [vehicleQueries.detail(VEHICLE_ID).queryKey])

    await invalidateRoots(queryClient, routerStub(), queryRoots.wateringPlan)

    expect(isInvalidated(queryClient, vehicleQueries.detail(VEHICLE_ID).queryKey)).toBe(false)
  })

  it('invalidates the plans and what the plan module derives from them on a status change', async () => {
    const queryClient = makeClient()
    const planKey = wateringPlanQueries.detail(PLAN_ID).queryKey
    seed(queryClient, [planKey, MODULE_KEY, vehicleQueries.detail(VEHICLE_ID).queryKey])

    await invalidateRoots(
      queryClient,
      routerStub(),
      planStatusQueryRoots({ invalidateOnStatusChange: [['module-thing']] }),
    )

    expect(isInvalidated(queryClient, planKey)).toBe(true)
    expect(isInvalidated(queryClient, MODULE_KEY)).toBe(true)
    expect(isInvalidated(queryClient, vehicleQueries.detail(VEHICLE_ID).queryKey)).toBe(false)
  })

  it('invalidates only the plans without a plan module', () => {
    expect(planStatusQueryRoots(undefined)).toEqual(queryRoots.wateringPlan)
  })

  it('only re-runs the route loaders when asked to', async () => {
    const queryClient = makeClient()

    const quiet = routerStub()
    await invalidateRoots(queryClient, quiet, queryRoots.vehicle)
    expect(quiet.invalidate).not.toHaveBeenCalled()

    const reloading = routerStub()
    await invalidateRoots(queryClient, reloading, queryRoots.vehicle, { reloadRoutes: true })
    expect(reloading.invalidate).toHaveBeenCalledTimes(1)
  })
})
