import { renderHook, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ModuleManifest } from '@/modules/types'
import { probeTarget, probeTargetManifest } from '@/test/targetManifest'

const { listSensors, probeSensorIds, activeManifests } = vi.hoisted(() => ({
  listSensors: vi.fn(),
  probeSensorIds: vi.fn(),
  activeManifests: { current: [] as ModuleManifest[] },
}))

vi.mock('@/api/backendApi', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/api/backendApi')>()),
  sensorApi: { listSensors },
}))

vi.mock('@/modules/useActiveModules', () => ({
  useActiveModules: () => ({ manifests: activeManifests.current, isActive: () => true }),
}))

import { useSensorList } from './useSensorList'

const Nothing = () => null

const probeManifest = probeTargetManifest('probe', [
  probeTarget('probe', 'probe', {
    listFilter: {
      param: 'probeIds',
      Control: Nothing,
      Chips: Nothing,
      sensorIds: (value) => ({
        queryKey: ['probe-sensor-ids', value],
        queryFn: () => probeSensorIds(value) as Promise<string[]>,
      }),
    },
  }),
])

const wrapper = ({ children }: { children: ReactNode }) => (
  <QueryClientProvider client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}>
    {children}
  </QueryClientProvider>
)

describe('useSensorList', () => {
  beforeEach(() => {
    listSensors.mockReset()
    probeSensorIds.mockReset()
    activeManifests.current = [probeManifest]
  })

  it('narrows the list to the sensor ids an active target filter resolves', async () => {
    probeSensorIds.mockResolvedValue(['s1', 's2'])
    listSensors.mockResolvedValue({ data: [], pagination: { totalRecords: 0 } })

    renderHook(() => useSensorList({ page: 1, probeIds: ['p-1'] }), { wrapper })

    await waitFor(() => expect(listSensors).toHaveBeenCalled())
    expect(probeSensorIds).toHaveBeenCalledWith(['p-1'])
    expect(listSensors).toHaveBeenCalledWith(expect.objectContaining({ id: ['s1', 's2'] }))
  })

  it('shows an empty list without calling the list endpoint when the filter matches no sensor', async () => {
    probeSensorIds.mockResolvedValue([])

    const { result } = renderHook(() => useSensorList({ page: 1, probeIds: ['p-1'] }), {
      wrapper,
    })

    await waitFor(() => expect(result.current.result).toEqual({ data: [] }))
    expect(listSensors).not.toHaveBeenCalled()
  })

  it('does not resolve sensor ids without a filter value', async () => {
    listSensors.mockResolvedValue({ data: [], pagination: { totalRecords: 0 } })

    renderHook(() => useSensorList({ page: 1 }), { wrapper })

    await waitFor(() => expect(listSensors).toHaveBeenCalled())
    expect(probeSensorIds).not.toHaveBeenCalled()
  })

  it('ignores the filter of a module that is not active', async () => {
    activeManifests.current = []
    listSensors.mockResolvedValue({ data: [], pagination: { totalRecords: 0 } })

    renderHook(() => useSensorList({ page: 1, probeIds: ['p-1'] }), { wrapper })

    await waitFor(() => expect(listSensors).toHaveBeenCalled())
    expect(probeSensorIds).not.toHaveBeenCalled()
    expect(listSensors).toHaveBeenCalledWith(expect.objectContaining({ id: undefined }))
  })
})
