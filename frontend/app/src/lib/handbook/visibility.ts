import type { ChapterMeta, HandbookPart } from './types'

export type ChapterFilter = (slug: string) => boolean

interface VisibilityInput {
  loggedIn: boolean
  /** Undefined while the modules request is pending or failed. */
  activeKeys: ReadonlySet<string> | undefined
}

/** Hiding is only a courtesy to logged-in users; a missing answer must never hide the handbook. */
export function chapterFilter(
  chapters: Record<string, ChapterMeta>,
  { loggedIn, activeKeys }: VisibilityInput,
): ChapterFilter {
  if (!loggedIn || !activeKeys) return () => true
  return (slug) => {
    const key = chapters[slug]?.module
    return key === undefined || activeKeys.has(key)
  }
}

export function visibleParts(parts: HandbookPart[], isVisible: ChapterFilter): HandbookPart[] {
  return parts
    .map((part) => ({ ...part, chapters: part.chapters.filter(isVisible) }))
    .filter((part) => part.chapters.length > 0)
}
