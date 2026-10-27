import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ReactNode } from 'react'
import { useWateringPlanForm } from './useWateringPlanForm'
import { Toaster } from '@green-ecolution/ui'
import { PlanStatus as WateringPlanStatus } from '@green-ecolution/backend-client'
import useStore from '@/store/store'
import type { WateringPlanForm } from '@/schema/wateringPlanSchema'
import { probeManifest } from '@/test/planningManifest'

vi.mock('@/api/backendApi', () => ({
  planApi: {
    createPlan: vi.fn(),
    updatePlan: vi.fn(),
  },
}))

const mockUseBlocker = vi.fn().mockReturnValue({
  proceed: vi.fn(),
  reset: vi.fn(),
  status: 'idle',
})

vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn().mockResolvedValue(undefined),
  useRouter: () => ({ invalidate: vi.fn().mockResolvedValue(undefined) }),
  // eslint-disable-next-line @typescript-eslint/no-unsafe-return
  useBlocker: (...args: unknown[]) => mockUseBlocker(...args),
}))

import { planApi } from '@/api/backendApi'
import type { PlanResponse } from '@/api/backendApi'

function createWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false, throwOnError: false },
    },
  })

  return ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      {children}
      <Toaster />
    </QueryClientProvider>
  )
}

const futureDate = new Date()
futureDate.setDate(futureDate.getDate() + 7)

const defaultInitForm: WateringPlanForm = {
  module: 'irrigation',
  date: futureDate,
  status: WateringPlanStatus.Planned,
  transporterId: 'transporter-uuid-1',
  trailerId: 'trailer-uuid-2',
  driverIds: ['550e8400-e29b-41d4-a716-446655440000'],
  stopIds: ['cluster-uuid-1', 'cluster-uuid-2'],
  description: '',
  transporterTankUsed: true,
  startPointName: 'Hof',
}

function createMockPlan(overrides: Partial<PlanResponse> = {}): PlanResponse {
  return {
    id: 'plan-uuid-1',
    date: futureDate.toISOString(),
    status: WateringPlanStatus.Planned,
    description: '',
    transporter: {
      id: 'transporter-uuid-1',
      numberPlate: 'HH-AB-1234',
      type: 'transporter',
      status: 'available',
      drivingLicense: 'B',
    },
    module: 'irrigation',
    stops: [],
    outcomes: [],
    userIds: [],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    ...overrides,
  } as unknown as PlanResponse
}

describe('useWateringPlanForm', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useStore.getState().clearAllFormDrafts()
  })

  afterEach(() => {
    useStore.getState().clearAllFormDrafts()
  })

  it('initializes form with provided default values', () => {
    const { result } = renderHook(
      () => useWateringPlanForm('create', { initForm: defaultInitForm }),
      { wrapper: createWrapper() },
    )

    expect(result.current.form.getValues('transporterId')).toBe('transporter-uuid-1')
    expect(result.current.form.getValues('status')).toBe(WateringPlanStatus.Planned)
    expect(result.current.form.getValues('driverIds')).toEqual([
      '550e8400-e29b-41d4-a716-446655440000',
    ])
    expect(result.current.form.getValues('stopIds')).toEqual(['cluster-uuid-1', 'cluster-uuid-2'])
  })

  it('returns form methods and mutation state', () => {
    const { result } = renderHook(
      () => useWateringPlanForm('create', { initForm: defaultInitForm }),
      { wrapper: createWrapper() },
    )

    expect(result.current.form).toBeDefined()
    expect(result.current.form.getValues).toBeDefined()
    expect(result.current.form.setValue).toBeDefined()
    expect(result.current.mutate).toBeDefined()
    expect(result.current.isError).toBe(false)
    expect(result.current.error).toBeNull()
  })

  it('creates the plan in the form module when mutationType is create', async () => {
    const mockResponse = createMockPlan()
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const createMock = vi.mocked(planApi.createPlan)
    createMock.mockResolvedValueOnce(mockResponse)

    const { result } = renderHook(
      () => useWateringPlanForm('create', { initForm: defaultInitForm }),
      { wrapper: createWrapper() },
    )

    act(() => {
      result.current.mutate(defaultInitForm)
    })

    await waitFor(() => {
      expect(createMock).toHaveBeenCalledTimes(1)
      expect(createMock).toHaveBeenCalledWith({
        planCreateRequest: {
          module: 'irrigation',
          date: futureDate.toISOString(),
          transporterId: 'transporter-uuid-1',
          trailerId: 'trailer-uuid-2',
          userIds: ['550e8400-e29b-41d4-a716-446655440000'],
          stops: ['cluster-uuid-1', 'cluster-uuid-2'],
          transporterCapacityUsed: true,
          description: '',
          startPointName: 'Hof',
        },
      })
    })
  })

  it('updates the plan when mutationType is update', async () => {
    const mockResponse = createMockPlan({
      id: 'plan-uuid-5',
      status: WateringPlanStatus.Active,
    })
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const updateMock = vi.mocked(planApi.updatePlan)
    updateMock.mockResolvedValueOnce(mockResponse)

    const updateInitForm = {
      ...defaultInitForm,
      status: WateringPlanStatus.Active,
    }

    const { result } = renderHook(
      () => useWateringPlanForm('update', { wateringPlanId: '5', initForm: updateInitForm }),
      { wrapper: createWrapper() },
    )

    act(() => {
      result.current.mutate(updateInitForm)
    })

    await waitFor(() => {
      expect(updateMock).toHaveBeenCalledTimes(1)
      expect(updateMock).toHaveBeenCalledWith({
        planId: '5',
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        planUpdateRequest: expect.objectContaining({
          status: WateringPlanStatus.Active,
          transporterId: 'transporter-uuid-1',
          stops: ['cluster-uuid-1', 'cluster-uuid-2'],
        }),
      })
    })
  })

  describe('draft management', () => {
    it('clears draft on successful mutation', async () => {
      const mockResponse = createMockPlan()
      // eslint-disable-next-line @typescript-eslint/unbound-method
      const createMock = vi.mocked(planApi.createPlan)
      createMock.mockResolvedValueOnce(mockResponse)

      // Pre-populate draft
      useStore.getState().setFormDraft('wateringplan-create', defaultInitForm)

      const { result } = renderHook(
        () => useWateringPlanForm('create', { initForm: defaultInitForm }),
        { wrapper: createWrapper() },
      )

      // Verify draft exists before mutation
      expect(useStore.getState().formDrafts['wateringplan-create']).toBeDefined()

      act(() => {
        result.current.mutate(defaultInitForm)
      })

      await waitFor(() => {
        expect(useStore.getState().formDrafts['wateringplan-create']).toBeUndefined()
      })
    })

    it('returns navigationBlocker with correct message', () => {
      const { result } = renderHook(
        () => useWateringPlanForm('create', { initForm: defaultInitForm }),
        { wrapper: createWrapper() },
      )

      expect(result.current.navigationBlocker).toBeDefined()
      expect(result.current.navigationBlocker.message).toContain('Einsatzplan')
    })

    it('lets the dirty form open the stop picker pages of the plan module', () => {
      const { result, rerender } = renderHook(
        () =>
          useWateringPlanForm('create', {
            initForm: defaultInitForm,
            planManifest: probeManifest('probe', { stopPickerPaths: ['/probe/pick'] }),
          }),
        { wrapper: createWrapper() },
      )
      act(() => {
        result.current.form.setValue('description', 'Neu', { shouldDirty: true })
      })
      rerender()

      const [{ shouldBlockFn }] = mockUseBlocker.mock.lastCall as [
        {
          shouldBlockFn: (args: {
            current: { pathname: string }
            next: { pathname: string }
          }) => boolean
        },
      ]
      const leave = (pathname: string) =>
        shouldBlockFn({ current: { pathname: '/plans/new' }, next: { pathname } })
      expect(leave('/probe/pick')).toBe(false)
      expect(leave('/vehicles')).toBe(true)
    })
  })
})
