import { queryOptions } from '@tanstack/react-query'
import type {
  ListResponseWasteBinResponse,
  ListWasteBinsRequest,
  WasteBinResponse,
} from '@green-ecolution/backend-client'
import { isValidUuid } from '@/api/queries'
import { wasteBinApi } from './client'

export { wasteRoots } from './roots'

// The API clamps per_page to 100, so a larger value would silently yield fewer rows per page.
const MARKER_PAGE_SIZE = 100

const fetchAllBins = async (): Promise<WasteBinResponse[]> => {
  const first = await wasteBinApi.listWasteBins({ page: 1, perPage: MARKER_PAGE_SIZE })
  const rest = await Promise.all(
    Array.from({ length: Math.max(first.pagination.totalPages - 1, 0) }, (_, i) =>
      wasteBinApi.listWasteBins({ page: i + 2, perPage: MARKER_PAGE_SIZE }),
    ),
  )
  return [first, ...rest].flatMap((page) => page.data)
}

const binsInStopOrder = (bins: WasteBinResponse[], ids: string[]): WasteBinResponse[] => {
  const byId = new Map(bins.map((bin) => [bin.id, bin]))
  return ids.flatMap((id) => byId.get(id) ?? [])
}

const MARKERS_KEY = ['waste-bins', 'markers'] as const
const MARKERS_STALE_TIME = 5 * 60_000

const list = (params?: ListWasteBinsRequest) =>
  queryOptions<ListResponseWasteBinResponse>({
    queryKey: ['waste-bins', 'list', params ?? {}],
    queryFn: () => wasteBinApi.listWasteBins(params),
  })

export const wasteBinQueries = {
  all: () => ['waste-bins'] as const,

  list,

  markers: () =>
    queryOptions<WasteBinResponse[]>({
      queryKey: MARKERS_KEY,
      queryFn: fetchAllBins,
      staleTime: MARKERS_STALE_TIME,
    }),

  /**
   * The bins behind a plan's stops, in stop order. Reads the visible bins the
   * map already holds; stops the reader may not see, or that were deleted, are
   * missing from the result.
   */
  byIds: (ids: string[]) =>
    queryOptions<WasteBinResponse[], Error, WasteBinResponse[]>({
      queryKey: MARKERS_KEY,
      queryFn: fetchAllBins,
      staleTime: MARKERS_STALE_TIME,
      select: (bins) => binsInStopOrder(bins, ids),
    }),

  detail: (id: string) =>
    queryOptions<WasteBinResponse>({
      queryKey: ['waste-bins', id],
      queryFn: () => wasteBinApi.getWasteBin({ wasteBinId: id }),
      enabled: isValidUuid(id),
    }),
}
