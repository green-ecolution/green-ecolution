import type { TreeClusterInListResponse } from '@green-ecolution/backend-client'
import { clusterApi } from './client'

const PAGE = 100

/**
 * The clusters behind a plan's stops, in stop order. Stops the reader may not
 * see, or that were deleted, are simply missing from the result.
 */
export async function loadStopClusters(stops: string[]): Promise<TreeClusterInListResponse[]> {
  const unique = [...new Set(stops)]
  const clusters = new Map<string, TreeClusterInListResponse>()
  for (let i = 0; i < unique.length; i += PAGE) {
    const page = await clusterApi.listClusters({ id: unique.slice(i, i + PAGE), perPage: PAGE })
    page.data.forEach((cluster) => clusters.set(cluster.id, cluster))
  }
  return stops.flatMap((id) => {
    const cluster = clusters.get(id)
    return cluster ? [cluster] : []
  })
}
