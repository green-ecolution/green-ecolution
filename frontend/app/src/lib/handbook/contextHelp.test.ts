import { createRouter } from '@tanstack/react-router'
import { describe, expect, it } from 'vitest'
import { routeTree } from '@/routeTree.gen'
import { chapterForRoute, normaliseRoute } from './contextHelp'
import { handbookIndex } from './index'

const declaredRoutes = () =>
  Object.values(handbookIndex.chapters).flatMap((chapter) =>
    chapter.routes.map((route) => ({ chapter: chapter.slug, route })),
  )

/** Every path the router itself can hand a match, index routes included. */
const routerFullPaths = (): string[] => {
  const router = createRouter({ routeTree, context: {} as never })
  const routes = Object.values(router.routesById) as { fullPath?: string }[]
  return routes
    .map((route) => route.fullPath)
    .filter((fullPath): fullPath is string => Boolean(fullPath))
}

describe('chapterForRoute', () => {
  it('maps a declared route to its chapter', () => {
    expect(chapterForRoute('/')).toBe('introduction')
  })

  it('maps the trailing-slash form a router match carries', () => {
    expect(chapterForRoute('/trees/')).toBe('trees')
  })

  it('maps a parameterised route', () => {
    expect(chapterForRoute('/trees/$treeId/')).toBe('trees')
  })

  it('maps a nested route', () => {
    expect(chapterForRoute('/settings/team/members/')).toBe('settings-team')
  })

  it('returns null for a route no chapter claims', () => {
    expect(chapterForRoute('/debug')).toBeNull()
  })

  it('returns null without a route', () => {
    expect(chapterForRoute(undefined)).toBeNull()
  })
})

describe('chapter route declarations', () => {
  it('only references routes that exist in the router', () => {
    const known = new Set(routerFullPaths().map(normaliseRoute))

    const unknown = declaredRoutes().filter(({ route }) => !known.has(normaliseRoute(route)))

    expect(unknown).toEqual([])
  })

  it('resolves every declared route in the form the router hands the link', () => {
    const byRoute = new Map<string, string[]>()
    for (const fullPath of routerFullPaths()) {
      const key = normaliseRoute(fullPath)
      byRoute.set(key, [...(byRoute.get(key) ?? []), fullPath])
    }

    const misses = declaredRoutes().flatMap(({ chapter, route }) =>
      (byRoute.get(normaliseRoute(route)) ?? []).flatMap((fullPath) =>
        chapterForRoute(fullPath) === chapter
          ? []
          : [`${fullPath} resolved to ${chapterForRoute(fullPath)}, expected ${chapter}`],
      ),
    )

    expect(misses).toEqual([])
  })

  it('never lets two chapters claim the same route', () => {
    const seen = new Map<string, string>()
    const clashes: string[] = []

    for (const chapter of Object.values(handbookIndex.chapters)) {
      for (const route of chapter.routes) {
        const key = normaliseRoute(route)
        const owner = seen.get(key)
        if (owner) clashes.push(`${key}: ${owner} and ${chapter.slug}`)
        else seen.set(key, chapter.slug)
      }
    }

    expect(clashes).toEqual([])
  })
})
