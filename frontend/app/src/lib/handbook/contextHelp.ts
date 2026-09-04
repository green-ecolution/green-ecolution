import { handbookIndex } from './index'

/**
 * A TanStack match carries the route's untrimmed `fullPath`, so the deepest
 * match on `/trees` is the index route's `/trees/`. Chapters declare the bare
 * form, so both sides are trimmed before they meet — except the root, which
 * would otherwise trim to an empty string.
 */
export function normaliseRoute(route: string): string {
  const trimmed = route.replace(/\/+$/, '')
  return trimmed === '' ? '/' : trimmed
}

const BY_ROUTE = new Map(
  Object.values(handbookIndex.chapters).flatMap((chapter) =>
    chapter.routes.map((route) => [normaliseRoute(route), chapter.slug] as const),
  ),
)

export function chapterForRoute(route: string | undefined): string | null {
  if (!route) return null
  return BY_ROUTE.get(normaliseRoute(route)) ?? null
}
