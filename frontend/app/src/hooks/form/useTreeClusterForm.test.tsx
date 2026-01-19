import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ReactNode } from 'react'
import { useTreeClusterForm } from './useTreeClusterForm'
import { Toaster } from '@green-ecolution/ui'
import { SoilCondition, WateringStatus } from '@green-ecolution/backend-client'

vi.mock('@/api/backendApi', () => ({
  clusterApi: {
    createTreeCluster: vi.fn(),
    updateTreeCluster: vi.fn(),
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

vi.mock('./usePersistForm', () => ({
  default: () => ({ clear: vi.fn() }),
}))

import { clusterApi } from '@/api/backendApi'
import type { TreeCluster } from '@green-ecolution/backend-client'

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

const defaultInitForm = {
  name: 'Test Cluster',
  address: 'Test Address 123',
  description: '',
  soilCondition: SoilCondition.TreeSoilConditionSandig,
  treeIds: [] as number[],
}

function createMockTreeCluster(overrides: Partial<TreeCluster> = {}): TreeCluster {
  return {
    id: 1,
    name: 'Test Cluster',
    address: 'Test Address 123',
    description: '',
    soilCondition: SoilCondition.TreeSoilConditionSandig,
    wateringStatus: WateringStatus.WateringStatusGood,
    trees: [],
    archived: false,
    region: null,
    latitude: 54.7937,
    longitude: 9.4469,
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    ...overrides,
  } as TreeCluster
}

describe('useTreeClusterForm', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    sessionStorage.clear()
  })

  afterEach(() => {
    sessionStorage.clear()
  })

  it('initializes form with provided default values', () => {
    const { result } = renderHook(
      () => useTreeClusterForm('create', { initForm: defaultInitForm }),
      { wrapper: createWrapper() },
    )

    expect(result.current.form.getValues('name')).toBe('Test Cluster')
    expect(result.current.form.getValues('address')).toBe('Test Address 123')
    expect(result.current.form.getValues('soilCondition')).toBe(
      SoilCondition.TreeSoilConditionSandig,
    )
    expect(result.current.form.getValues('treeIds')).toEqual([])
  })

  it('returns form methods and mutation state', () => {
    const { result } = renderHook(
      () => useTreeClusterForm('create', { initForm: defaultInitForm }),
      { wrapper: createWrapper() },
    )

    expect(result.current.form).toBeDefined()
    expect(result.current.form.getValues).toBeDefined()
    expect(result.current.form.setValue).toBeDefined()
    expect(result.current.mutate).toBeDefined()
    expect(result.current.isError).toBe(false)
    expect(result.current.error).toBeNull()
  })

  it('calls createTreeCluster API when mutationType is create', async () => {
    const mockResponse = createMockTreeCluster()
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const createMock = vi.mocked(clusterApi.createTreeCluster)
    createMock.mockResolvedValueOnce(mockResponse)

    const { result } = renderHook(
      () => useTreeClusterForm('create', { initForm: defaultInitForm }),
      { wrapper: createWrapper() },
    )

    act(() => {
      result.current.mutate({
        name: 'Test Cluster',
        address: 'Test Address 123',
        description: '',
        soilCondition: SoilCondition.TreeSoilConditionSandig,
        treeIds: [],
      })
    })

    await waitFor(() => {
      expect(createMock).toHaveBeenCalledTimes(1)
      expect(createMock).toHaveBeenCalledWith({
        body: {
          name: 'Test Cluster',
          address: 'Test Address 123',
          description: '',
          soilCondition: SoilCondition.TreeSoilConditionSandig,
          treeIds: [],
        },
      })
    })
  })

  it('calls updateTreeCluster API when mutationType is update', async () => {
    const mockResponse = createMockTreeCluster({
      id: 5,
      name: 'Updated Cluster',
      address: 'Updated Address',
      soilCondition: SoilCondition.TreeSoilConditionLehmig,
    })
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const updateMock = vi.mocked(clusterApi.updateTreeCluster)
    updateMock.mockResolvedValueOnce(mockResponse)

    const updateInitForm = {
      ...defaultInitForm,
      name: 'Updated Cluster',
      address: 'Updated Address',
      soilCondition: SoilCondition.TreeSoilConditionLehmig,
    }

    const { result } = renderHook(
      () => useTreeClusterForm('update', { clusterId: '5', initForm: updateInitForm }),
      { wrapper: createWrapper() },
    )

    act(() => {
      result.current.mutate({
        name: 'Updated Cluster',
        address: 'Updated Address',
        description: '',
        soilCondition: SoilCondition.TreeSoilConditionLehmig,
        treeIds: [],
      })
    })

    await waitFor(() => {
      expect(updateMock).toHaveBeenCalledTimes(1)
      expect(updateMock).toHaveBeenCalledWith({
        clusterId: 5,
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
        body: expect.objectContaining({
          name: 'Updated Cluster',
          address: 'Updated Address',
        }),
      })
    })
  })

  describe('map trees changed flag', () => {
    it('detects trees changed flag for create mutation', () => {
      sessionStorage.setItem('create-cluster-trees-changed', 'true')

      renderHook(() => useTreeClusterForm('create', { initForm: defaultInitForm }), {
        wrapper: createWrapper(),
      })

      expect(mockUseBlocker).toHaveBeenCalledWith(
        expect.objectContaining({
          shouldBlockFn: expect.any(Function) as unknown,
        }),
      )
    })

    it('detects trees changed flag for update mutation', () => {
      sessionStorage.setItem('update-cluster-trees-changed', 'true')

      renderHook(
        () => useTreeClusterForm('update', { clusterId: '1', initForm: defaultInitForm }),
        {
          wrapper: createWrapper(),
        },
      )

      expect(mockUseBlocker).toHaveBeenCalledWith(
        expect.objectContaining({
          shouldBlockFn: expect.any(Function) as unknown,
        }),
      )
    })

    it('does not detect trees changed when flag is not set', () => {
      renderHook(() => useTreeClusterForm('create', { initForm: defaultInitForm }), {
        wrapper: createWrapper(),
      })

      expect(mockUseBlocker).toHaveBeenCalled()
    })

    it('clears trees changed flags on successful mutation', async () => {
      sessionStorage.setItem('create-cluster-trees-changed', 'true')
      sessionStorage.setItem('update-cluster-trees-changed', 'true')

      const mockResponse = createMockTreeCluster()
      // eslint-disable-next-line @typescript-eslint/unbound-method
      const createMock = vi.mocked(clusterApi.createTreeCluster)
      createMock.mockResolvedValueOnce(mockResponse)

      const { result } = renderHook(
        () => useTreeClusterForm('create', { initForm: defaultInitForm }),
        { wrapper: createWrapper() },
      )

      act(() => {
        result.current.mutate({
          name: 'Test Cluster',
          address: 'Test Address 123',
          description: '',
          soilCondition: SoilCondition.TreeSoilConditionSandig,
          treeIds: [],
        })
      })

      await waitFor(() => {
        expect(sessionStorage.getItem('create-cluster-trees-changed')).toBeNull()
        expect(sessionStorage.getItem('update-cluster-trees-changed')).toBeNull()
      })
    })
  })
})
