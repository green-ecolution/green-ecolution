import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'

const api = vi.hoisted(() => ({
  createPlanComment: vi.fn(),
  updatePlanComment: vi.fn(),
  deletePlanComment: vi.fn(),
  createTargetComment: vi.fn(),
  updateTargetComment: vi.fn(),
  deleteTargetComment: vi.fn(),
}))

vi.mock('@/api/backendApi', () => ({ configuration: {}, commentApi: api }))
vi.mock('@/hooks/createToast', () => ({ default: () => vi.fn() }))

import { commentQueryKey, type CommentSubject } from '@/api/queries'
import { useCommentMutations } from './useCommentMutations'

const PLAN_ID = '0190a8e9-7c4f-7000-8000-000000000001'
const CLUSTER_ID = '0190a8e9-7c4f-7000-8000-000000000002'

const planSubject: CommentSubject = { kind: 'plan', id: PLAN_ID }
const targetSubject: CommentSubject = {
  kind: 'target',
  target: { module: 'irrigation', kind: 'cluster', id: CLUSTER_ID },
}

const setup = (subject: CommentSubject) => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, staleTime: 60_000 }, mutations: { retry: false } },
  })
  queryClient.setQueryData(commentQueryKey(planSubject), { seeded: true })
  queryClient.setQueryData(commentQueryKey(targetSubject), { seeded: true })
  const wrapper = ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  )
  const hook = renderHook(() => useCommentMutations(subject), { wrapper })
  const invalidated = (s: CommentSubject) =>
    queryClient.getQueryState(commentQueryKey(s))?.isInvalidated === true
  return { ...hook, invalidated }
}

type Op = 'create' | 'update' | 'remove'

const run = async (hook: ReturnType<typeof setup>, op: Op) => {
  const m = hook.result.current[op]
  if (op === 'create') m.mutate('Text' as never)
  else if (op === 'update') m.mutate({ commentId: 'c1', body: 'Neu' } as never)
  else m.mutate('c1' as never)
  await waitFor(() => expect(hook.result.current[op].isSuccess).toBe(true))
}

describe('useCommentMutations', () => {
  beforeEach(() => {
    Object.values(api).forEach((fn) => fn.mockReset().mockResolvedValue({}))
  })

  describe.each<[Op, keyof typeof api, keyof typeof api, unknown, unknown]>([
    [
      'create',
      'createPlanComment',
      'createTargetComment',
      { planId: PLAN_ID, createCommentRequest: { body: 'Text' } },
      {
        module: 'irrigation',
        kind: 'cluster',
        targetId: CLUSTER_ID,
        createCommentRequest: { body: 'Text' },
      },
    ],
    [
      'update',
      'updatePlanComment',
      'updateTargetComment',
      { planId: PLAN_ID, commentId: 'c1', updateCommentRequest: { body: 'Neu' } },
      {
        module: 'irrigation',
        kind: 'cluster',
        targetId: CLUSTER_ID,
        commentId: 'c1',
        updateCommentRequest: { body: 'Neu' },
      },
    ],
    [
      'remove',
      'deletePlanComment',
      'deleteTargetComment',
      { planId: PLAN_ID, commentId: 'c1' },
      { module: 'irrigation', kind: 'cluster', targetId: CLUSTER_ID, commentId: 'c1' },
    ],
  ])('%s', (op, planMethod, targetMethod, planParams, targetParams) => {
    it('uses the plan route and invalidates only the plan comments', async () => {
      const hook = setup(planSubject)
      await run(hook, op)

      expect(api[planMethod]).toHaveBeenCalledWith(planParams)
      expect(api[targetMethod]).not.toHaveBeenCalled()
      expect(hook.invalidated(planSubject)).toBe(true)
      expect(hook.invalidated(targetSubject)).toBe(false)
    })

    it('uses the target route and invalidates only the target comments', async () => {
      const hook = setup(targetSubject)
      await run(hook, op)

      expect(api[targetMethod]).toHaveBeenCalledWith(targetParams)
      expect(api[planMethod]).not.toHaveBeenCalled()
      expect(hook.invalidated(targetSubject)).toBe(true)
      expect(hook.invalidated(planSubject)).toBe(false)
    })
  })
})
