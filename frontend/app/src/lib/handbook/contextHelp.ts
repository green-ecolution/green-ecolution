import { handbookIndex } from './index'

const BY_ROUTE = new Map(
  Object.values(handbookIndex.chapters).flatMap((chapter) =>
    chapter.routes.map((route) => [route, chapter.slug] as const),
  ),
)

export function chapterForRoute(route: string | undefined): string | null {
  if (!route) return null
  return BY_ROUTE.get(route) ?? null
}
