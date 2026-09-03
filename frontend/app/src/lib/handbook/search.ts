import { handbookIndex } from './index'
import type { SearchEntry } from './types'

export interface SearchHit {
  slug: string
  anchor: string
  chapterTitle: string
  sectionTitle: string
  excerpt: string
}

const EXCERPT_RADIUS = 60
const MIN_QUERY_LENGTH = 2

export function isSearchableQuery(query: string): boolean {
  return query.trim().length >= MIN_QUERY_LENGTH
}

function excerpt(text: string, at: number): string {
  const start = Math.max(0, at - EXCERPT_RADIUS)
  const end = Math.min(text.length, at + EXCERPT_RADIUS)
  return `${start > 0 ? '… ' : ''}${text.slice(start, end).trim()}${end < text.length ? ' …' : ''}`
}

export function searchHandbook(entries: SearchEntry[], query: string): SearchHit[] {
  if (!isSearchableQuery(query)) return []
  const needle = query.trim().toLowerCase()

  return entries
    .flatMap((entry) => {
      const inTitle = entry.sectionTitle.toLowerCase().indexOf(needle)
      const inText = entry.text.toLowerCase().indexOf(needle)
      if (inTitle < 0 && inText < 0) return []

      return [
        {
          rank: inTitle >= 0 ? 0 : 1,
          hit: {
            slug: entry.slug,
            anchor: entry.anchor,
            chapterTitle: handbookIndex.chapters[entry.slug]?.title ?? entry.slug,
            sectionTitle: entry.sectionTitle,
            excerpt:
              inText >= 0 ? excerpt(entry.text, inText) : entry.text.slice(0, EXCERPT_RADIUS * 2),
          },
        },
      ]
    })
    .sort((a, b) => a.rank - b.rank)
    .map(({ hit }) => hit)
}
