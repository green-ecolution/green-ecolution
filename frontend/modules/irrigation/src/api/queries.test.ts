import { describe, it, expect, vi, beforeEach } from 'vitest'
import type {
  ListResponseTreeClusterInListResponse,
  ListResponseTreeResponse,
} from '@green-ecolution/backend-client'
import { clusterQueries, treeQueries } from './queries'
import type { Tree, TreeCluster } from './client'

vi.mock('./client', () => ({
  treeApi: {
    listTrees: vi.fn(),
    getTree: vi.fn(),
  },
  clusterApi: {
    listClusters: vi.fn(),
    getCluster: vi.fn(),
  },
}))

import { treeApi, clusterApi } from './client'

describe('Irrigation Query Functions', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Tree Queries', () => {
    describe('treeQueries.list', () => {
      it('returns correct query options for fetching all trees', () => {
        const options = treeQueries.list()

        expect(options.queryKey).toContain('trees')
        expect(options.queryFn).toBeDefined()
      })

      it('includes pagination params in query key', () => {
        const options = treeQueries.list({ page: 2 })

        expect(options.queryKey).toEqual(['trees', 'list', { page: 2 }])
      })

      it('calls treeApi.listTrees when queryFn is executed', async () => {
        const mockResponse = { data: [] } as unknown as ListResponseTreeResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(treeApi.listTrees).mockResolvedValueOnce(mockResponse)

        const options = treeQueries.list()
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(treeApi.listTrees).toHaveBeenCalledWith(undefined)
        expect(result).toEqual(mockResponse)
      })

      it('passes params to treeApi.listTrees', async () => {
        const mockResponse = { data: [] } as unknown as ListResponseTreeResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(treeApi.listTrees).mockResolvedValueOnce(mockResponse)

        const params = { page: 2 }
        const options = treeQueries.list(params)
        await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(treeApi.listTrees).toHaveBeenCalledWith(params)
      })

      it('includes q in the query key when provided', () => {
        const options = treeQueries.list({ page: 1, perPage: 10, q: 'Eiche' })
        expect(options.queryKey).toEqual(['trees', 'list', { page: 1, perPage: 10, q: 'Eiche' }])
      })

      it('includes filter params in the query key', () => {
        const options = treeQueries.list({ page: 1, perPage: 10, wateringStatus: ['good'] })
        expect(options.queryKey).toEqual([
          'trees',
          'list',
          { page: 1, perPage: 10, wateringStatus: ['good'] },
        ])
      })

      it('passes q to treeApi.listTrees', async () => {
        const mockResponse = { data: [] } as unknown as ListResponseTreeResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(treeApi.listTrees).mockResolvedValueOnce(mockResponse)

        const options = treeQueries.list({ page: 1, perPage: 10, q: 'Eiche' })
        await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(treeApi.listTrees).toHaveBeenCalledWith({ page: 1, perPage: 10, q: 'Eiche' })
      })
    })

    describe('treeQueries.detail', () => {
      it('returns correct query options for fetching single tree', () => {
        const options = treeQueries.detail('123')

        expect(options.queryKey).toEqual(['tree', '123'])
        expect(options.queryFn).toBeDefined()
      })

      it('calls treeApi.getTree with correct id', async () => {
        const mockTree = { id: 'tree-uuid-1', latitude: 54.0, longitude: 9.0 } as unknown as Tree
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(treeApi.getTree).mockResolvedValueOnce(mockTree)

        const options = treeQueries.detail('tree-uuid-1')
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(treeApi.getTree).toHaveBeenCalledWith({ treeId: 'tree-uuid-1' })
        expect(result).toEqual(mockTree)
      })

      it('passes string id directly to API', async () => {
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(treeApi.getTree).mockResolvedValueOnce({} as Tree)

        const options = treeQueries.detail('tree-uuid-2')
        await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(treeApi.getTree).toHaveBeenCalledWith({ treeId: 'tree-uuid-2' })
      })
    })
  })

  describe('TreeCluster Queries', () => {
    describe('clusterQueries.list', () => {
      it('returns correct query options for fetching all clusters', () => {
        const options = clusterQueries.list()

        expect(options.queryKey).toContain('treeclusters')
        expect(options.queryFn).toBeDefined()
      })

      it('calls clusterApi.listClusters when queryFn is executed', async () => {
        const mockResponse = { data: [] } as unknown as ListResponseTreeClusterInListResponse
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(clusterApi.listClusters).mockResolvedValueOnce(mockResponse)

        const options = clusterQueries.list()
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(clusterApi.listClusters).toHaveBeenCalledWith(undefined)
        expect(result).toEqual(mockResponse)
      })
    })

    describe('clusterQueries.detail', () => {
      it('returns correct query options for fetching single cluster', () => {
        const options = clusterQueries.detail('99')

        expect(options.queryKey).toEqual(['treecluster', '99'])
        expect(options.queryFn).toBeDefined()
      })

      it('calls clusterApi.getCluster with correct id', async () => {
        const mockCluster = {
          id: 'cluster-uuid-1',
          name: 'Test Cluster',
        } as unknown as TreeCluster
        // eslint-disable-next-line @typescript-eslint/unbound-method
        vi.mocked(clusterApi.getCluster).mockResolvedValueOnce(mockCluster)

        const options = clusterQueries.detail('cluster-uuid-1')
        const result = await options.queryFn!({} as never)

        // eslint-disable-next-line @typescript-eslint/unbound-method
        expect(clusterApi.getCluster).toHaveBeenCalledWith({ clusterId: 'cluster-uuid-1' })
        expect(result).toEqual(mockCluster)
      })
    })
  })
})
