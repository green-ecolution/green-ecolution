import { describe, it, expect, vi, beforeEach } from 'vitest'
import type { ReactNode } from 'react'
import { renderHook, waitFor } from '@testing-library/react'
import { PlanStatus as WateringPlanStatus } from '@green-ecolution/backend-client'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { QueryKey } from '@tanstack/react-query'
import { wateringPlanQueries } from '@/api/queries'
import { useWateringPlanBoardMutations } from './useWateringPlanBoardMutations'
import { planInList } from '@/test/plans'

const updatePlan = vi.fn()
vi.mock('@/api/backendApi', () => ({
  planApi: {
    updatePlan: (...args: unknown[]) => updatePlan(...args) as unknown,
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
const HIDDEN_CLUSTER_ID = '0190a8e9-7c4f-7000-8000-000000000006'

// Module query keys stand in for whatever a plan module derives from its plans.
const CLUSTER_DETAIL: QueryKey = ['treecluster', CLUSTER_ID]
const TREE_DETAIL: QueryKey = ['tree', TREE_ID]
const PLAN_DETAIL: QueryKey = wateringPlanQueries.detail(PLAN_ID).queryKey
const planning = { invalidateOnStatusChange: [['treecluster'], ['tree']] }

const plan = planInList({
  id: PLAN_ID,
  status: WateringPlanStatus.Active,
  transporter: { id: '0190a8e9-7c4f-7000-8000-000000000004' } as never,
  stops: [CLUSTER_ID, HIDDEN_CLUSTER_ID],
})

const outcomes = [{ stop: CLUSTER_ID, completed: true, amount: 100 }]

/** Mirrors main.tsx: a 60s staleTime is what makes a missed invalidation visible. */
function renderBoardMutations() {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { staleTime: 60_000, retry: false }, mutations: { retry: false } },
  })
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  )
  const view = renderHook(() => useWateringPlanBoardMutations(planning), { wrapper })

  // Cached views a user visited before the mutation.
  ;[CLUSTER_DETAIL, TREE_DETAIL, PLAN_DETAIL].forEach((key) =>
    queryClient.setQueryData(key, { seeded: true }),
  )

  return { ...view, queryClient }
}

describe('useWateringPlanBoardMutations', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    updatePlan.mockResolvedValue({ id: PLAN_ID })
  })

  it('invalidates what the plan module names after finishing a plan', async () => {
    const { result, queryClient } = renderBoardMutations()
    const isInvalidated = (key: QueryKey) => queryClient.getQueryState(key)?.isInvalidated === true

    result.current.finishPlan.mutate({ plan, outcomes })

    await waitFor(() => expect(isInvalidated(CLUSTER_DETAIL)).toBe(true))
    expect(isInvalidated(TREE_DETAIL)).toBe(true)
    expect(isInvalidated(PLAN_DETAIL)).toBe(true)
  })

  it('finishes a plan with the reported outcomes and the hidden stops as done', async () => {
    const { result } = renderBoardMutations()

    result.current.finishPlan.mutate({ plan, outcomes })

    await waitFor(() => expect(updatePlan).toHaveBeenCalledTimes(1))
    expect(updatePlan).toHaveBeenCalledWith({
      planId: PLAN_ID,
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      planUpdateRequest: expect.objectContaining({
        status: WateringPlanStatus.Finished,
        stops: [CLUSTER_ID, HIDDEN_CLUSTER_ID],
        outcomes: [
          { stop: CLUSTER_ID, completed: true, amount: 100 },
          { stop: HIDDEN_CLUSTER_ID, completed: true, amount: null },
        ],
      }),
    })
  })

  it('keeps stops the user cannot see when a plan is started from the board', async () => {
    const { result } = renderBoardMutations()

    result.current.startPlan.mutate({ ...plan, status: WateringPlanStatus.Planned })

    await waitFor(() => expect(updatePlan).toHaveBeenCalledTimes(1))
    expect(updatePlan).toHaveBeenCalledWith({
      planId: PLAN_ID,
      // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
      planUpdateRequest: expect.objectContaining({
        status: WateringPlanStatus.Active,
        stops: [CLUSTER_ID, HIDDEN_CLUSTER_ID],
      }),
    })
  })

  it('re-runs the route loaders that loader-based detail pages read from', async () => {
    const { result } = renderBoardMutations()

    result.current.finishPlan.mutate({ plan, outcomes })

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
