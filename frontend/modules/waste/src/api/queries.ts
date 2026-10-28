import { queryOptions } from '@tanstack/react-query'
import type {
  ListResponseWasteBinResponse,
  ListWasteBinsRequest,
  WasteBinResponse,
} from '@green-ecolution/backend-client'
import { isValidUuid } from '@/api/queries'
import { wasteBinApi } from './client'

export { wasteRoots } from './roots'

export const wasteBinQueries = {
  all: () => ['waste-bins'] as const,

  list: (params?: ListWasteBinsRequest) =>
    queryOptions<ListResponseWasteBinResponse>({
      queryKey: ['waste-bins', 'list', params ?? {}],
      queryFn: () => wasteBinApi.listWasteBins(params),
    }),

  detail: (id: string) =>
    queryOptions<WasteBinResponse>({
      queryKey: ['waste-bins', id],
      queryFn: () => wasteBinApi.getWasteBin({ wasteBinId: id }),
      enabled: isValidUuid(id),
    }),
}
