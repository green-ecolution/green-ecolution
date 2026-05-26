import { treeApi } from '@/api/backendApi'
import {
  Alert,
  AlertContent,
  AlertDescription,
  AlertTitle,
  Badge,
  Button,
  Loading,
  cn,
} from '@green-ecolution/ui'
import type { TreeResponse } from '@green-ecolution/backend-client'
import { keepPreviousData, useInfiniteQuery } from '@tanstack/react-query'
import { Check, Search, TreeDeciduous } from 'lucide-react'
import { useEffect, useRef } from 'react'

interface SensorTreeSearchResultsProps {
  q: string
  selectedTreeId: string | null
  onSelect: (treeId: string) => void
}

const PER_PAGE = 20

const SensorTreeSearchResults = ({ q, selectedTreeId, onSelect }: SensorTreeSearchResultsProps) => {
  const trimmed = q.trim()
  const enabled = trimmed.length > 0
  const sentinelRef = useRef<HTMLDivElement | null>(null)

  const { data, isLoading, isError, refetch, hasNextPage, fetchNextPage, isFetchingNextPage } =
    useInfiniteQuery({
      queryKey: ['trees', 'search', trimmed],
      queryFn: ({ pageParam }) =>
        treeApi.listTrees({ page: pageParam, perPage: PER_PAGE, q: trimmed }),
      initialPageParam: 1,
      getNextPageParam: (lastPage, allPages) => {
        const total = lastPage.pagination?.totalRecords ?? 0
        const loaded = allPages.reduce((sum, p) => sum + (p.data?.length ?? 0), 0)
        return loaded < total ? allPages.length + 1 : undefined
      },
      enabled,
      placeholderData: keepPreviousData,
    })

  useEffect(() => {
    const el = sentinelRef.current
    if (!el || !hasNextPage || isFetchingNextPage) return
    const obs = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting) void fetchNextPage()
      },
      { rootMargin: '0px 0px 200px 0px' },
    )
    obs.observe(el)
    return () => obs.disconnect()
  }, [hasNextPage, isFetchingNextPage, fetchNextPage])

  if (!enabled) {
    return (
      <div className="flex flex-col items-center justify-center gap-2 py-10 text-dark-600">
        <Search className="size-6 text-dark-400" aria-hidden />
        <p className="text-sm">Tippe Baumnummer oder Baumart ein.</p>
      </div>
    )
  }

  if (isLoading) {
    return <Loading className="py-10 justify-center" label="Bäume werden gesucht…" />
  }

  if (isError) {
    return (
      <Alert variant="destructive">
        <AlertContent>
          <AlertTitle>Suche fehlgeschlagen</AlertTitle>
          <AlertDescription>Die Baumsuche ist fehlgeschlagen.</AlertDescription>
        </AlertContent>
        <Button variant="outline" size="sm" onClick={() => void refetch()}>
          Erneut versuchen
        </Button>
      </Alert>
    )
  }

  const items = data?.pages.flatMap((p) => p.data ?? []) ?? []
  const total = data?.pages[data.pages.length - 1]?.pagination?.totalRecords ?? items.length

  if (items.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center gap-2 py-10 text-dark-600">
        <Search className="size-6 text-dark-400" aria-hidden />
        <p className="text-sm">Keine Bäume gefunden für „{trimmed}".</p>
        <p className="text-xs text-dark-500">Prüfe Schreibweise oder Baumnummer.</p>
      </div>
    )
  }

  return (
    <div className="flex flex-col gap-2">
      <p className="text-xs text-dark-600">
        {items.length} von {total} Treffern
      </p>
      <ul
        className="flex flex-col gap-2"
        role="radiogroup"
        aria-label="Baum aus Suchergebnis wählen"
      >
        {items.map((tree) => (
          <li key={tree.id}>
            <ResultRow
              tree={tree}
              selected={tree.id === selectedTreeId}
              onSelect={() => onSelect(tree.id)}
            />
          </li>
        ))}
      </ul>
      <div ref={sentinelRef} aria-hidden className="h-1" />
      {isFetchingNextPage && (
        <Loading className="py-3 justify-center" label="Weitere Bäume werden geladen…" />
      )}
    </div>
  )
}

function ResultRow({
  tree,
  selected,
  onSelect,
}: {
  tree: TreeResponse
  selected: boolean
  onSelect: () => void
}) {
  const isAssigned = tree.sensor != null
  return (
    <button
      type="button"
      onClick={isAssigned ? undefined : onSelect}
      disabled={isAssigned}
      aria-disabled={isAssigned || undefined}
      aria-pressed={!isAssigned && selected}
      className={cn(
        'relative w-full text-left rounded-xl border bg-white p-4 shadow-cards transition',
        !isAssigned && 'hover:bg-green-dark-50/50',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-green-dark focus-visible:ring-offset-2',
        isAssigned && 'opacity-70 cursor-not-allowed',
        !isAssigned && selected
          ? 'border-green-dark ring-2 ring-green-dark/20 bg-green-dark-50/30'
          : 'border-dark-100',
      )}
    >
      <div className="flex items-start gap-3">
        <div
          className={cn(
            'mt-0.5 flex size-5 shrink-0 items-center justify-center rounded-full border-2',
            !isAssigned && selected
              ? 'border-green-dark bg-green-dark text-white'
              : 'border-dark-200 bg-white',
          )}
        >
          {!isAssigned && selected && <Check className="size-3" strokeWidth={3} />}
        </div>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2 min-w-0">
            <TreeDeciduous className="size-4 shrink-0 text-green-dark" aria-hidden />
            <span className="font-semibold text-sm truncate">{tree.species}</span>
          </div>
          <div className="mt-2 flex flex-wrap items-center gap-x-3 gap-y-1 text-sm text-dark-800">
            <span className="font-mono text-xs text-dark-600">{tree.number}</span>
            {isAssigned && (
              <Badge variant="muted" size="default" aria-label="Sensor zugeordnet">
                Sensor zugeordnet
              </Badge>
            )}
          </div>
        </div>
      </div>
    </button>
  )
}

export default SensorTreeSearchResults
export type { SensorTreeSearchResultsProps }
