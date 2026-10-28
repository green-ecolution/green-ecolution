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

export const wasteBinQueries = {
  all: () => ['waste-bins'] as const,

  list: (params?: ListWasteBinsRequest) =>
    queryOptions<ListResponseWasteBinResponse>({
      queryKey: ['waste-bins', 'list', params ?? {}],
      queryFn: () => wasteBinApi.listWasteBins(params),
    }),

  markers: () =>
    queryOptions<WasteBinResponse[]>({
      queryKey: ['waste-bins', 'markers'],
      queryFn: fetchAllBins,
      staleTime: 5 * 60_000,
    }),

  detail: (id: string) =>
    queryOptions<WasteBinResponse>({
      queryKey: ['waste-bins', id],
      queryFn: () => wasteBinApi.getWasteBin({ wasteBinId: id }),
      enabled: isValidUuid(id),
    }),
}
