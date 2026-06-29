import { treeApi } from '@/api/backendApi'
import type { TreeResponse } from '@green-ecolution/backend-client'
import { keepPreviousData, useInfiniteQuery } from '@tanstack/react-query'

const PER_PAGE = 20

export const useTreeSearch = (q: string, showAll = false) => {
  const trimmed = q.trim()
  const enabled = showAll || trimmed.length > 0

  const query = useInfiniteQuery({
    queryKey: ['trees', 'search', trimmed],
    queryFn: ({ pageParam }) =>
      treeApi.listTrees({ page: pageParam, perPage: PER_PAGE, q: trimmed || undefined }),
    initialPageParam: 1,
    getNextPageParam: (lastPage, allPages) => {
      const total = lastPage.pagination?.totalRecords ?? 0
      const loaded = allPages.reduce((sum, p) => sum + (p.data?.length ?? 0), 0)
      return loaded < total ? allPages.length + 1 : undefined
    },
    enabled,
    placeholderData: keepPreviousData,
    // Avoid a full multi-page refetch when the query is re-enabled (toggling
    // "show all"); that refetch races with fetchNextPage and stalls paging.
    staleTime: 5 * 60 * 1000,
  })

  const items: TreeResponse[] = query.data?.pages.flatMap((p) => p.data ?? []) ?? []
  const total =
    query.data?.pages[query.data.pages.length - 1]?.pagination?.totalRecords ?? items.length

  return { ...query, enabled, trimmed, items, total }
}
