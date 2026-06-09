import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import type { TreeClusterResponse } from '@/api/backendApi'
import { useClusterPanelEdit } from './useClusterPanelEdit'

const updateCluster = vi.fn()
vi.mock('@/api/backendApi', async (importOriginal) => {
  const actual = await importOriginal<typeof import('@/api/backendApi')>()
  return {
    ...actual,
    clusterApi: { updateCluster: (...args: unknown[]) => updateCluster(...args) as unknown },
  }
})
vi.mock('@green-ecolution/domain-wasm', () => ({
  clusterDraftResolver: () => (values: unknown) => Promise.resolve({ values, errors: {} }),
}))
const showToast = vi.fn()
vi.mock('@/hooks/createToast', () => ({ default: () => showToast }))

const cluster = {
  id: '11111111-1111-4111-8111-111111111111',
  name: 'Hafenspitze',
  address: 'Schiffbrücke 12',
  description: 'Notiz',
  soilCondition: 'sandig',
  trees: [{ id: 't1' }, { id: 't2' }],
} as unknown as TreeClusterResponse

const wrapper = ({ children }: { children: ReactNode }) => {
  const qc = new QueryClient({
    defaultOptions: { queries: { retry: false }, mutations: { retry: false } },
  })
  return <QueryClientProvider client={qc}>{children}</QueryClientProvider>
}

describe('useClusterPanelEdit', () => {
  beforeEach(() => vi.clearAllMocks())

  it('submits name + existing treeIds and calls onSaved', async () => {
    updateCluster.mockResolvedValue({ ...cluster, name: 'Neuer Name' })
    const onSaved = vi.fn()
    const { result } = renderHook(() => useClusterPanelEdit(cluster, { onSaved }), { wrapper })

    act(() => result.current.form.setValue('name', 'Neuer Name'))
    await act(async () => {
      await result.current.onSubmit()
    })

    await waitFor(() => expect(onSaved).toHaveBeenCalledTimes(1))
    expect(updateCluster).toHaveBeenCalledWith({
      clusterId: cluster.id,
      treeClusterUpdateRequest: expect.objectContaining({
        name: 'Neuer Name',
        address: 'Schiffbrücke 12',
        soilCondition: 'sandig',
        treeIds: ['t1', 't2'],
      }) as unknown,
    })
  })
})
