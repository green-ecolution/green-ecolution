import { treeApi } from '@/api/backendApi'
import type { TreeResponse } from '@green-ecolution/backend-client'
import { keepPreviousData, useInfiniteQuery } from '@tanstack/react-query'

const PER_PAGE = 20

export const useTreeSearch = (q: string) => {
  const trimmed = q.trim()
  const enabled = trimmed.length > 0

  const query = useInfiniteQuery({
    queryKey: ['trees', 'search', trimmed],
    queryFn: ({ pageParam }) => treeApi.listTrees({ page: pageParam, perPage: PER_PAGE, q: trimmed }),
    initialPageParam: 1,
    getNextPageParam: (lastPage, allPages) => {
      const total = lastPage.pagination?.totalRecords ?? 0
      const loaded = allPages.reduce((sum, p) => sum + (p.data?.length ?? 0), 0)
      return loaded < total ? allPages.length + 1 : undefined
    },
    enabled,
    placeholderData: keepPreviousData,
  })

  const items: TreeResponse[] = query.data?.pages.flatMap((p) => p.data ?? []) ?? []
  const total =
    query.data?.pages[query.data.pages.length - 1]?.pagination?.totalRecords ?? items.length

  return { ...query, enabled, trimmed, items, total }
}
