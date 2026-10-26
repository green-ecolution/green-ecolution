import { describe, expect, it, vi, beforeEach } from 'vitest'
import type { ListResponseTreeClusterInListResponse } from '@green-ecolution/backend-client'
import { loadStopClusters } from './plans'

vi.mock('./client', () => ({ clusterApi: { listClusters: vi.fn() } }))

const { clusterApi } = await import('./client')
// eslint-disable-next-line @typescript-eslint/unbound-method
const listClusters = vi.mocked(clusterApi.listClusters)

const page = (...ids: string[]) =>
  ({
    data: ids.map((id) => ({ id, name: `Cluster ${id}` })),
  }) as unknown as ListResponseTreeClusterInListResponse

describe('loadStopClusters', () => {
  beforeEach(() => vi.clearAllMocks())

  it('resolves stops to clusters in stop order and skips the ones it cannot see', async () => {
    listClusters.mockResolvedValueOnce(page('c2', 'c1'))

    const clusters = await loadStopClusters(['c1', 'hidden', 'c2'])

    expect(clusters.map((c) => c.id)).toEqual(['c1', 'c2'])
    expect(listClusters).toHaveBeenCalledWith({ id: ['c1', 'hidden', 'c2'], perPage: 100 })
  })

  it('pages through more than a hundred stops', async () => {
    const stops = Array.from({ length: 150 }, (_, i) => `c${i}`)
    listClusters.mockResolvedValueOnce(page(...stops.slice(0, 100)))
    listClusters.mockResolvedValueOnce(page(...stops.slice(100)))

    const clusters = await loadStopClusters(stops)

    expect(listClusters).toHaveBeenCalledTimes(2)
    expect(clusters).toHaveLength(150)
  })

  it('asks once per distinct stop', async () => {
    listClusters.mockResolvedValueOnce(page('c1'))

    await loadStopClusters(['c1', 'c1'])

    expect(listClusters).toHaveBeenCalledWith({ id: ['c1'], perPage: 100 })
  })
})
