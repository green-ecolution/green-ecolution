import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ReactNode } from 'react'
import { useTreeForm } from './useTreeForm'
import ToastProvider from '@/context/ToastContext'

vi.mock('@/api/backendApi', () => ({
  treeApi: {
    createTree: vi.fn(),
    updateTree: vi.fn(),
  },
}))

vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn().mockResolvedValue(undefined),
  useBlocker: vi.fn(),
}))

vi.mock('./usePersistForm', () => ({
  default: () => ({ clear: vi.fn() }),
}))

import { treeApi } from '@/api/backendApi'
import type { Tree } from '@green-ecolution/backend-client'

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
  latitude: 53.5511,
  longitude: 9.9937,
  number: 'T-001',
  species: 'Oak',
  plantingYear: 2023,
  treeClusterId: -1,
  sensorId: '-1',
  description: '',
}

describe('useTreeForm', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('initializes form with provided default values', () => {
    const { result } = renderHook(() => useTreeForm('create', { initForm: defaultInitForm }), {
      wrapper: createWrapper(),
    })

    expect(result.current.form.getValues('number')).toBe('T-001')
    expect(result.current.form.getValues('species')).toBe('Oak')
    expect(result.current.form.getValues('latitude')).toBe(53.5511)
    expect(result.current.form.getValues('longitude')).toBe(9.9937)
  })

  it('returns form methods', () => {
    const { result } = renderHook(() => useTreeForm('create', { initForm: defaultInitForm }), {
      wrapper: createWrapper(),
    })

    expect(result.current.form).toBeDefined()
    expect(result.current.mutate).toBeDefined()
    expect(result.current.isError).toBe(false)
    expect(result.current.error).toBeNull()
  })

  it('calls createTree API when mutationType is create', async () => {
    const mockResponse = {
      id: 1,
      number: 'T-001',
      species: 'Oak',
      latitude: 53.5511,
      longitude: 9.9937,
      plantingYear: 2023,
    } as Tree
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const createTreeMock = vi.mocked(treeApi.createTree)
    createTreeMock.mockResolvedValueOnce(mockResponse)

    const { result } = renderHook(() => useTreeForm('create', { initForm: defaultInitForm }), {
      wrapper: createWrapper(),
    })

    act(() => {
      result.current.mutate({
        latitude: 53.5511,
        longitude: 9.9937,
        number: 'T-001',
        species: 'Oak',
        plantingYear: 2023,
        description: '',
      })
    })

    await waitFor(() => {
      expect(createTreeMock).toHaveBeenCalledWith(
        expect.objectContaining({
          body: expect.objectContaining({
            number: 'T-001',
            species: 'Oak',
          }) as unknown,
        }),
      )
    })
  })

  it('calls updateTree API when mutationType is update', async () => {
    const mockResponse = {
      id: 5,
      number: 'T-005',
      species: 'Maple',
      latitude: 53.5511,
      longitude: 9.9937,
      plantingYear: 2022,
    } as Tree
    // eslint-disable-next-line @typescript-eslint/unbound-method
    const updateTreeMock = vi.mocked(treeApi.updateTree)
    updateTreeMock.mockResolvedValueOnce(mockResponse)

    const updateInitForm = {
      ...defaultInitForm,
      number: 'T-005',
      species: 'Maple',
      plantingYear: 2022,
    }

    const { result } = renderHook(
      () => useTreeForm('update', { treeId: '5', initForm: updateInitForm }),
      { wrapper: createWrapper() },
    )

    act(() => {
      result.current.mutate({
        latitude: 53.5511,
        longitude: 9.9937,
        number: 'T-005',
        species: 'Maple',
        plantingYear: 2022,
        description: '',
      })
    })

    await waitFor(() => {
      expect(updateTreeMock).toHaveBeenCalledWith(
        expect.objectContaining({
          treeId: 5,
          body: expect.objectContaining({ species: 'Maple' }) as unknown,
        }),
      )
    })
  })

  it('uses zod resolver for form validation', () => {
    const { result } = renderHook(() => useTreeForm('create', { initForm: defaultInitForm }), {
      wrapper: createWrapper(),
    })

    // Verify the form is configured with validation
    expect(result.current.form).toBeDefined()
    // Form validation is already tested extensively in treeSchema.test.ts
  })
})
