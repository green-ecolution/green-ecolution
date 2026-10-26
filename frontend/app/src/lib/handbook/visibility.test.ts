import { describe, expect, it } from 'vitest'
import { chapterFilter, visibleParts } from './visibility'
import type { ChapterMeta, HandbookPart } from './types'

const chapter = (slug: string, module?: string): ChapterMeta => ({
  slug,
  title: slug,
  part: 'p',
  summary: '',
  routes: [],
  sections: [],
  ...(module && { module }),
})

const chapters = {
  intro: chapter('intro'),
  trees: chapter('trees', 'irrigation'),
  waste: chapter('waste', 'waste'),
}

describe('chapterFilter', () => {
  it('shows everything to a visitor who is not logged in', () => {
    const visible = chapterFilter(chapters, { loggedIn: false, activeKeys: undefined })

    expect(['intro', 'trees', 'waste'].map(visible)).toEqual([true, true, true])
  })

  it('hides chapters of inactive modules from a logged-in user', () => {
    const visible = chapterFilter(chapters, { loggedIn: true, activeKeys: new Set(['irrigation']) })

    expect(['intro', 'trees', 'waste'].map(visible)).toEqual([true, true, false])
  })

  it('hides every module chapter when no module is active', () => {
    const visible = chapterFilter(chapters, { loggedIn: true, activeKeys: new Set() })

    expect(['intro', 'trees', 'waste'].map(visible)).toEqual([true, false, false])
  })

  it('shows everything while the active modules are not known yet', () => {
    const visible = chapterFilter(chapters, { loggedIn: true, activeKeys: undefined })

    expect(['intro', 'trees', 'waste'].map(visible)).toEqual([true, true, true])
  })

  it('treats an unknown slug as visible', () => {
    const visible = chapterFilter(chapters, { loggedIn: true, activeKeys: new Set() })

    expect(visible('nope')).toBe(true)
  })
})

describe('visibleParts', () => {
  const parts: HandbookPart[] = [
    { id: 'a', title: 'A', chapters: ['intro', 'trees'] },
    { id: 'b', title: 'B', chapters: ['waste'] },
  ]

  it('drops hidden chapters and parts left without chapters', () => {
    expect(visibleParts(parts, (slug) => slug === 'intro')).toEqual([
      { id: 'a', title: 'A', chapters: ['intro'] },
    ])
  })
})
