import { describe, it, expect, vi, beforeEach } from 'vitest'
import type { ReactNode } from 'react'
import { renderHook, waitFor } from '@testing-library/react'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { QueryKey } from '@tanstack/react-query'
import { clusterQueries, treeQueries, wateringPlanQueries } from '@/api/queries'
import { useWateringPlanBoardMutations } from './useWateringPlanBoardMutations'
import type { WateringPlanInList } from '@/api/backendApi'

const updateWateringPlan = vi.fn()
vi.mock('@/api/backendApi', () => ({
  wateringPlanApi: {
    updateWateringPlan: (...args: unknown[]) => updateWateringPlan(...args) as unknown,
  },
}))

const routerInvalidate = vi.fn().mockResolvedValue(undefined)
vi.mock('@tanstack/react-router', () => ({
  useRouter: () => ({ invalidate: routerInvalidate }),
}))

vi.mock('@/hooks/createToast', () => ({ default: () => vi.fn() }))

const CLUSTER_ID = '0190a8e9-7c4f-7000-8000-000000000001'
const TREE_ID = '0190a8e9-7c4f-7000-8000-000000000002'
const PLAN_ID = '0190a8e9-7c4f-7000-8000-000000000003'

const CLUSTER_DETAIL: QueryKey = clusterQueries.detail(CLUSTER_ID).queryKey
const CLUSTER_LIST: QueryKey = clusterQueries.list().queryKey
const TREE_DETAIL: QueryKey = treeQueries.detail(TREE_ID).queryKey
const PLAN_DETAIL: QueryKey = wateringPlanQueries.detail(PLAN_ID).queryKey

const plan = {
  id: PLAN_ID,
  date: '2026-08-19T00:00:00Z',
  description: '',
  status: WateringPlanStatus.Active,
  transporter: { id: '0190a8e9-7c4f-7000-8000-000000000004' },
  treeclusters: [{ id: CLUSTER_ID }],
  userIds: [],
} as unknown as WateringPlanInList

const evaluation = [{ wateringPlanId: PLAN_ID, treeClusterId: CLUSTER_ID, consumedWater: 100 }]

/** Mirrors main.tsx: a 60s staleTime is what makes a missed invalidation visible. */
function renderBoardMutations() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { staleTime: 60_000, retry: false }, mutations: { retry: false } },
  })
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  )
  const view = renderHook(() => useWateringPlanBoardMutations(), { wrapper })

  // Cached views a user visited before the mutation.
  ;[CLUSTER_DETAIL, CLUSTER_LIST, TREE_DETAIL, PLAN_DETAIL].forEach((key) =>
    queryClient.setQueryData(key, { seeded: true }),
  )

  return { ...view, queryClient }
}

describe('useWateringPlanBoardMutations', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    updateWateringPlan.mockResolvedValue({ id: PLAN_ID })
  })

  it('invalidates clusters and trees after finishing a plan', async () => {
    const { result, queryClient } = renderBoardMutations()
    const isInvalidated = (key: QueryKey) => queryClient.getQueryState(key)?.isInvalidated === true

    result.current.finishPlan.mutate({ plan, evaluation })

    await waitFor(() => expect(isInvalidated(CLUSTER_DETAIL)).toBe(true))
    expect(isInvalidated(CLUSTER_LIST)).toBe(true)
    expect(isInvalidated(TREE_DETAIL)).toBe(true)
    expect(isInvalidated(PLAN_DETAIL)).toBe(true)
  })

  it('re-runs the route loaders that loader-based detail pages read from', async () => {
    const { result } = renderBoardMutations()

    result.current.finishPlan.mutate({ plan, evaluation })

    await waitFor(() => expect(routerInvalidate).toHaveBeenCalled())
  })

  it('keeps a user assignment scoped to watering plans', async () => {
    const { result, queryClient } = renderBoardMutations()
    const isInvalidated = (key: QueryKey) => queryClient.getQueryState(key)?.isInvalidated === true

    result.current.assignUsers.mutate({ plan, userIds: ['u1'] })

    await waitFor(() => expect(isInvalidated(PLAN_DETAIL)).toBe(true))
    expect(isInvalidated(CLUSTER_DETAIL)).toBe(false)
    expect(isInvalidated(TREE_DETAIL)).toBe(false)
  })
})
