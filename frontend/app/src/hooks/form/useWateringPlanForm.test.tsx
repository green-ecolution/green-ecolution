import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ReactNode } from 'react'
import { useWaterinPlanForm } from './useWateringPlanForm'
import ToastProvider from '@/context/ToastContext'
import { WateringPlanStatus } from '@green-ecolution/backend-client'

vi.mock('@/api/backendApi', () => ({
  wateringPlanApi: {
    createWateringPlan: vi.fn(),
    updateWateringPlan: vi.fn(),
  },
}))

vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn().mockResolvedValue(undefined),
}))

vi.mock('./usePersistForm', () => ({
  default: () => ({ clear: vi.fn() }),
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
      <ToastProvider>{children}</ToastProvider>
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
  cluserIds: [1, 2],
  description: '',
}

describe('useWaterinPlanForm', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('initializes form with provided default values', () => {
    const { result } = renderHook(
      () => useWaterinPlanForm('create', { initForm: defaultInitForm }),
      { wrapper: createWrapper() },
    )

    expect(result.current.form.getValues('transporterId')).toBe(1)
    expect(result.current.form.getValues('status')).toBe(
      WateringPlanStatus.WateringPlanStatusPlanned,
    )
    expect(result.current.form.getValues('driverIds')).toEqual([
      '550e8400-e29b-41d4-a716-446655440000',
    ])
    expect(result.current.form.getValues('cluserIds')).toEqual([1, 2])
  })

  it('returns form methods', () => {
    const { result } = renderHook(
      () => useWaterinPlanForm('create', { initForm: defaultInitForm }),
      { wrapper: createWrapper() },
    )

    expect(result.current.form).toBeDefined()
    expect(result.current.mutate).toBeDefined()
    expect(result.current.isError).toBe(false)
    expect(result.current.error).toBeNull()
  })

  it('calls createWateringPlan API when mutationType is create', async () => {
    const mockResponse = {
      id: 1,
      date: futureDate.toISOString(),
      status: WateringPlanStatus.WateringPlanStatusPlanned,
    } as WateringPlan
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const createMock = vi.mocked(wateringPlanApi.createWateringPlan)
    createMock.mockResolvedValueOnce(mockResponse)

    const { result } = renderHook(
      () => useWaterinPlanForm('create', { initForm: defaultInitForm }),
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
      expect(createMock).toHaveBeenCalledWith(
        expect.objectContaining({
          body: expect.objectContaining({
            transporterId: 1,
          }) as unknown,
        }),
      )
    })
  })

  it('calls updateWateringPlan API when mutationType is update', async () => {
    const mockResponse = {
      id: 5,
      date: futureDate.toISOString(),
      status: WateringPlanStatus.WateringPlanStatusActive,
    } as WateringPlan
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const updateMock = vi.mocked(wateringPlanApi.updateWateringPlan)
    updateMock.mockResolvedValueOnce(mockResponse)

    const updateInitForm = {
      ...defaultInitForm,
      status: WateringPlanStatus.WateringPlanStatusActive,
    }

    const { result } = renderHook(
      () => useWaterinPlanForm('update', { wateringPlanId: '5', initForm: updateInitForm }),
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
      expect(updateMock).toHaveBeenCalledWith(
        expect.objectContaining({
          id: '5',
          body: expect.objectContaining({
            status: WateringPlanStatus.WateringPlanStatusActive,
          }) as unknown,
        }),
      )
    })
  })

  it('passes userIds and clusterIds correctly to API', async () => {
    const mockResponse = {
      id: 1,
    } as WateringPlan
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const createMock = vi.mocked(wateringPlanApi.createWateringPlan)
    createMock.mockResolvedValueOnce(mockResponse)

    const { result } = renderHook(
      () => useWaterinPlanForm('create', { initForm: defaultInitForm }),
      { wrapper: createWrapper() },
    )

    act(() => {
      result.current.mutate({
        date: futureDate.toISOString(),
        transporterId: 1,
        userIds: ['550e8400-e29b-41d4-a716-446655440000', '550e8400-e29b-41d4-a716-446655440001'],
        treeClusterIds: [1, 2, 3],
        description: 'Test description',
      })
    })

    await waitFor(() => {
      expect(createMock).toHaveBeenCalledWith(
        expect.objectContaining({
          body: expect.objectContaining({
            userIds: [
              '550e8400-e29b-41d4-a716-446655440000',
              '550e8400-e29b-41d4-a716-446655440001',
            ],
            treeClusterIds: [1, 2, 3],
          }) as unknown,
        }),
      )
    })
  })
})
