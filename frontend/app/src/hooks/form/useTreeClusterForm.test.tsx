import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ReactNode } from 'react'
import { useTreeClusterForm } from './useTreeClusterForm'
import ToastProvider from '@/context/ToastContext'
import { SoilCondition } from '@green-ecolution/backend-client'

vi.mock('@/api/backendApi', () => ({
  clusterApi: {
    createTreeCluster: vi.fn(),
    updateTreeCluster: vi.fn(),
  },
}))

vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn().mockResolvedValue(undefined),
  useBlocker: vi.fn(),
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
      <ToastProvider>{children}</ToastProvider>
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

describe('useTreeClusterForm', () => {
  beforeEach(() => {
    vi.clearAllMocks()
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

  it('returns form methods', () => {
    const { result } = renderHook(
      () => useTreeClusterForm('create', { initForm: defaultInitForm }),
      { wrapper: createWrapper() },
    )

    expect(result.current.form).toBeDefined()
    expect(result.current.mutate).toBeDefined()
    expect(result.current.isError).toBe(false)
    expect(result.current.error).toBeNull()
  })

  it('calls createTreeCluster API when mutationType is create', async () => {
    const mockResponse = {
      id: 1,
      name: 'Test Cluster',
      address: 'Test Address 123',
      soilCondition: SoilCondition.TreeSoilConditionSandig,
    } as TreeCluster
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
      expect(createMock).toHaveBeenCalledWith(
        expect.objectContaining({
          body: expect.objectContaining({
            name: 'Test Cluster',
            address: 'Test Address 123',
          }) as unknown,
        }),
      )
    })
  })

  it('calls updateTreeCluster API when mutationType is update', async () => {
    const mockResponse = {
      id: 5,
      name: 'Updated Cluster',
      address: 'Updated Address',
      soilCondition: SoilCondition.TreeSoilConditionLehmig,
    } as TreeCluster
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
      expect(updateMock).toHaveBeenCalledWith(
        expect.objectContaining({
          clusterId: 5,
          body: expect.objectContaining({ name: 'Updated Cluster' }) as unknown,
        }),
      )
    })
  })

  it('passes treeIds correctly to API', async () => {
    const mockResponse = {
      id: 1,
      name: 'Cluster with Trees',
      treeIds: [1, 2, 3],
    } as TreeCluster
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const createMock = vi.mocked(clusterApi.createTreeCluster)
    createMock.mockResolvedValueOnce(mockResponse)

    const initFormWithTrees = {
      ...defaultInitForm,
      treeIds: [1, 2, 3],
    }

    const { result } = renderHook(
      () => useTreeClusterForm('create', { initForm: initFormWithTrees }),
      { wrapper: createWrapper() },
    )

    act(() => {
      result.current.mutate({
        name: 'Cluster with Trees',
        address: 'Test Address',
        description: '',
        soilCondition: SoilCondition.TreeSoilConditionSandig,
        treeIds: [1, 2, 3],
      })
    })

    await waitFor(() => {
      expect(createMock).toHaveBeenCalledWith(
        expect.objectContaining({
          body: expect.objectContaining({
            treeIds: [1, 2, 3],
          }) as unknown,
        }),
      )
    })
  })
})
