import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ReactNode } from 'react'
import { useWateringPlanForm } from './useWateringPlanForm'
import { Toaster } from '@green-ecolution/ui'
import { WateringPlanStatus } from '@green-ecolution/backend-client'
import useStore from '@/store/store'

vi.mock('@/api/backendApi', () => ({
  wateringPlanApi: {
    createWateringPlan: vi.fn(),
    updateWateringPlan: vi.fn(),
  },
}))

const mockUseBlocker = vi.fn().mockReturnValue({
  proceed: vi.fn(),
  reset: vi.fn(),
  status: 'idle',
})

vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn().mockResolvedValue(undefined),
  // eslint-disable-next-line @typescript-eslint/no-unsafe-return
  useBlocker: (...args: unknown[]) => mockUseBlocker(...args),
}))

import { wateringPlanApi } from '@/api/backendApi'
import type { WateringPlan } from '@green-ecolution/backend-client'

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

const defaultInitForm = {
  date: futureDate,
  status: WateringPlanStatus.WateringPlanStatusPlanned,
  transporterId: 1,
  trailerId: 2,
  driverIds: ['550e8400-e29b-41d4-a716-446655440000'],
  clusterIds: [1, 2],
  description: '',
}

function createMockWateringPlan(overrides: Partial<WateringPlan> = {}): WateringPlan {
  return {
    id: 1,
    date: futureDate.toISOString(),
    status: WateringPlanStatus.WateringPlanStatusPlanned,
    description: '',
    transporter: {
      id: 1,
      numberPlate: 'HH-AB-1234',
      type: 'transporter',
      status: 'available',
      drivingLicense: 'B',
    },
    treeclusters: [],
    userIds: [],
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    ...overrides,
  } as WateringPlan
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

    expect(result.current.form.getValues('transporterId')).toBe(1)
    expect(result.current.form.getValues('status')).toBe(
      WateringPlanStatus.WateringPlanStatusPlanned,
    )
    expect(result.current.form.getValues('driverIds')).toEqual([
      '550e8400-e29b-41d4-a716-446655440000',
    ])
    expect(result.current.form.getValues('clusterIds')).toEqual([1, 2])
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

  it('calls createWateringPlan API when mutationType is create', async () => {
    const mockResponse = createMockWateringPlan()
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const createMock = vi.mocked(wateringPlanApi.createWateringPlan)
    createMock.mockResolvedValueOnce(mockResponse)

    const { result } = renderHook(
      () => useWateringPlanForm('create', { initForm: defaultInitForm }),
      { wrapper: createWrapper() },
    )

    act(() => {
      result.current.mutate({
        date: futureDate.toISOString(),
        transporterId: 1,
        userIds: ['550e8400-e29b-41d4-a716-446655440000'],
        treeClusterIds: [1, 2],
        description: '',
      })
    })

    await waitFor(() => {
      expect(createMock).toHaveBeenCalledTimes(1)
      expect(createMock).toHaveBeenCalledWith({
        body: {
          date: futureDate.toISOString(),
          transporterId: 1,
          userIds: ['550e8400-e29b-41d4-a716-446655440000'],
          treeClusterIds: [1, 2],
          description: '',
        },
      })
    })
  })

  it('calls updateWateringPlan API when mutationType is update', async () => {
    const mockResponse = createMockWateringPlan({
      id: 5,
      status: WateringPlanStatus.WateringPlanStatusActive,
    })
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const updateMock = vi.mocked(wateringPlanApi.updateWateringPlan)
    updateMock.mockResolvedValueOnce(mockResponse)

    const updateInitForm = {
      ...defaultInitForm,
      status: WateringPlanStatus.WateringPlanStatusActive,
    }

    const { result } = renderHook(
      () => useWateringPlanForm('update', { wateringPlanId: '5', initForm: updateInitForm }),
      { wrapper: createWrapper() },
    )

    act(() => {
      result.current.mutate({
        date: futureDate.toISOString(),
        status: WateringPlanStatus.WateringPlanStatusActive,
        transporterId: 1,
        userIds: ['550e8400-e29b-41d4-a716-446655440000'],
        treeClusterIds: [1, 2],
        description: '',
        cancellationNote: '',
      })
    })

    await waitFor(() => {
      expect(updateMock).toHaveBeenCalledTimes(1)
      expect(updateMock).toHaveBeenCalledWith({
        id: '5',
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        body: expect.objectContaining({
          status: WateringPlanStatus.WateringPlanStatusActive,
          transporterId: 1,
        }),
      })
    })
  })

  describe('draft management', () => {
    it('clears draft on successful mutation', async () => {
      const mockResponse = createMockWateringPlan()
      // eslint-disable-next-line @typescript-eslint/unbound-method
      const createMock = vi.mocked(wateringPlanApi.createWateringPlan)
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
        result.current.mutate({
          date: futureDate.toISOString(),
          transporterId: 1,
          userIds: ['550e8400-e29b-41d4-a716-446655440000'],
          treeClusterIds: [1, 2],
          description: '',
        })
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
  })
})
