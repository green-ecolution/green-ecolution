import { renderHook, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'

const { listSensors, listClusterSensorIds } = vi.hoisted(() => ({
  listSensors: vi.fn(),
  listClusterSensorIds: vi.fn(),
}))

vi.mock('@/api/backendApi', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/api/backendApi')>()),
  sensorApi: { listSensors },
}))

vi.mock('../../../../../modules/irrigation/src/api/client', async (importOriginal) => ({
  ...(await importOriginal<typeof import('../../../../../modules/irrigation/src/api/client')>()),
  clusterApi: { listClusterSensorIds },
}))

import { useSensorList } from './useSensorList'

const wrapper = ({ children }: { children: ReactNode }) => (
  <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
    {children}
  </QueryClientProvider>
)

describe('useSensorList', () => {
  beforeEach(() => {
    listSensors.mockReset()
    listClusterSensorIds.mockReset()
  })

  it('resolves the cluster filter to sensor ids and passes them to the list', async () => {
    listClusterSensorIds.mockImplementation(({ clusterId }: { clusterId: string }) =>
      Promise.resolve(clusterId === 'c1' ? ['s1', 's2'] : ['s2', 's3']),
    )
    listSensors.mockResolvedValue({ data: [], pagination: { totalRecords: 0 } })

    renderHook(() => useSensorList({ page: 1, clusterIds: ['c2', 'c1'] }), { wrapper })

    await waitFor(() => expect(listSensors).toHaveBeenCalled())
    expect(listSensors).toHaveBeenCalledWith(expect.objectContaining({ id: ['s1', 's2', 's3'] }))
  })

  it('shows an empty list without calling the list endpoint when the clusters have no sensors', async () => {
    listClusterSensorIds.mockResolvedValue([])

    const { result } = renderHook(() => useSensorList({ page: 1, clusterIds: ['c1'] }), {
      wrapper,
    })

    await waitFor(() => expect(result.current.result).toEqual({ data: [] }))
    expect(listSensors).not.toHaveBeenCalled()
  })

  it('does not resolve cluster ids without a cluster filter', async () => {
    listSensors.mockResolvedValue({ data: [], pagination: { totalRecords: 0 } })

    renderHook(() => useSensorList({ page: 1 }), { wrapper })

    await waitFor(() => expect(listSensors).toHaveBeenCalled())
    expect(listClusterSensorIds).not.toHaveBeenCalled()
  })
})
