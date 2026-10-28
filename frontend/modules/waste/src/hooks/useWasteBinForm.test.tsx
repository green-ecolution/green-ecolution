import type { ReactNode } from 'react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { act, renderHook, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { Toaster } from '@green-ecolution/ui'
import { ResponseError } from '@green-ecolution/backend-client'
import useStore from '@/store/store'
import { wasteBinApi } from '../api/client'
import { wasteBinFixtures } from '../test/handlers'

const navigate = vi.fn().mockResolvedValue(undefined)
vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => navigate,
  useRouter: () => ({ invalidate: vi.fn().mockResolvedValue(undefined) }),
  useBlocker: () => ({ proceed: vi.fn(), reset: vi.fn(), status: 'idle' }),
}))

const { useWasteBinForm } = await import('./useWasteBinForm')

const wrapper = ({ children }: { children: ReactNode }) => {
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  return (
    <QueryClientProvider client={queryClient}>
      {children}
      <Toaster />
    </QueryClientProvider>
  )
}

const draft = {
  number: 'B-0100',
  latitude: 54.78,
  longitude: 9.43,
  volumeLiters: 1100,
  depthCm: 120,
  organizationId: 'org-1',
}

describe('useWasteBinForm', () => {
  beforeEach(() => {
    vi.restoreAllMocks()
    navigate.mockClear()
    useStore.getState().clearAllFormDrafts()
  })

  afterEach(() => useStore.getState().clearAllFormDrafts())

  it('creates a bin with its owning organization and opens it', async () => {
    const create = vi
      .spyOn(wasteBinApi, 'createWasteBin')
      .mockResolvedValue({ ...wasteBinFixtures[0], id: 'bin-new' })
    const { result } = renderHook(() => useWasteBinForm('create', { initForm: draft }), {
      wrapper,
    })

    act(() => result.current.mutate(draft))

    await waitFor(() => expect(create).toHaveBeenCalledWith({ wasteBinCreateRequest: draft }))
    await waitFor(() =>
      expect(navigate).toHaveBeenCalledWith(
        expect.objectContaining({
          to: '/waste-bins/$wasteBinId',
          params: { wasteBinId: 'bin-new' },
        }),
      ),
    )
  })

  it('updates a bin without the organization, which only a transfer changes', async () => {
    const update = vi.spyOn(wasteBinApi, 'updateWasteBin').mockResolvedValue(wasteBinFixtures[0])
    const { result } = renderHook(
      () => useWasteBinForm('update', { wasteBinId: 'bin-1', initForm: draft }),
      { wrapper },
    )

    act(() => result.current.mutate(draft))

    await waitFor(() =>
      expect(update).toHaveBeenCalledWith({
        wasteBinId: 'bin-1',
        wasteBinUpdateRequest: {
          number: 'B-0100',
          latitude: 54.78,
          longitude: 9.43,
          volumeLiters: 1100,
          depthCm: 120,
        },
      }),
    )
  })

  it('names a taken bin number in its own words', async () => {
    vi.spyOn(console, 'error').mockImplementation(() => undefined)
    vi.spyOn(wasteBinApi, 'createWasteBin').mockRejectedValue(
      new ResponseError(
        new Response(JSON.stringify({ error: 'taken', code: 'waste_bin.number_taken' }), {
          status: 409,
          headers: { 'Content-Type': 'application/json' },
        }),
      ),
    )
    const { result } = renderHook(() => useWasteBinForm('create', { initForm: draft }), {
      wrapper,
    })

    act(() => result.current.mutate(draft))

    await waitFor(() =>
      expect(result.current.error?.message).toBe(
        'Diese Behälternummer ist in der Organisation bereits vergeben.',
      ),
    )
  })
})
