import { createRouter } from '@tanstack/react-router'
import { describe, expect, it } from 'vitest'
import { routeTree } from '@/routeTree.gen'
import { chapterForRoute } from './contextHelp'
import { handbookIndex } from './index'

describe('chapterForRoute', () => {
  it('maps a declared route to its chapter', () => {
    expect(chapterForRoute('/')).toBe('introduction')
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
    const router = createRouter({ routeTree, context: {} as never })
    const known = new Set(Object.keys(router.routesByPath))

    const declared = Object.values(handbookIndex.chapters).flatMap((chapter) =>
      chapter.routes.map((route) => ({ chapter: chapter.slug, route })),
    )
    const unknown = declared.filter(({ route }) => !known.has(route))

    expect(unknown).toEqual([])
  })

  it('never lets two chapters claim the same route', () => {
    const seen = new Map<string, string>()
    const clashes: string[] = []

    for (const chapter of Object.values(handbookIndex.chapters)) {
      for (const route of chapter.routes) {
        const owner = seen.get(route)
        if (owner) clashes.push(`${route}: ${owner} and ${chapter.slug}`)
        else seen.set(route, chapter.slug)
      }
    }

    expect(clashes).toEqual([])
  })
})
