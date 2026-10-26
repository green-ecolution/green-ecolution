import { describe, expect, it } from 'vitest'
import { chapterNeighbours, firstChapter, handbookIndex, loadChapter } from './index'
import type { HandbookIndex } from './types'

describe('handbook index', () => {
  it('matches the declared manifest type', () => {
    const typed: HandbookIndex = handbookIndex
    expect(typed.parts.length).toBeGreaterThan(0)
  })

  it('lists every chapter under exactly one part', () => {
    const listed = handbookIndex.parts.flatMap((part) => part.chapters)

    expect([...listed].sort()).toEqual(Object.keys(handbookIndex.chapters).sort())
    expect(new Set(listed).size).toBe(listed.length)
  })

  it('loads a chapter body', async () => {
    const chapter = await loadChapter('introduction')

    expect(chapter.slug).toBe('introduction')
    expect(chapter.blocks.length).toBeGreaterThan(0)
  })

  it('rejects an unknown chapter', async () => {
    await expect(loadChapter('nope')).rejects.toThrow(/unknown chapter/)
  })
})

describe('chapterNeighbours', () => {
  it('points at the chapters around one in the middle of a part', () => {
    const { previous, next } = chapterNeighbours('trees')

    expect(previous).toEqual({
      slug: 'map',
      title: 'Die Karte',
      partTitle: 'Grünflächen',
      entersNewPart: false,
    })
    expect(next).toEqual({
      slug: 'treecluster',
      title: 'Bewässerungsgruppen',
      partTitle: 'Grünflächen',
      entersNewPart: false,
    })
  })

  it('marks a step that leaves the current part', () => {
    expect(chapterNeighbours('sensor-installation').next).toEqual({
      slug: 'settings-profile',
      title: 'Profil',
      partTitle: 'Verwaltung',
      entersNewPart: true,
    })
    expect(chapterNeighbours('settings-profile').previous?.entersNewPart).toBe(true)
  })

  it('has no predecessor for the first chapter', () => {
    const { previous, next } = chapterNeighbours('introduction')

    expect(previous).toBeNull()
    expect(next?.slug).toBe('getting-started')
  })

  it('has no successor for the last chapter', () => {
    const { previous, next } = chapterNeighbours('troubleshooting')

    expect(previous?.slug).toBe('glossary')
    expect(next).toBeNull()
  })

  it('steps over chapters the filter hides', () => {
    const { previous, next } = chapterNeighbours('trees', (slug) => slug !== 'treecluster')

    expect(previous?.slug).toBe('map')
    expect(next?.slug).toBe('watering-plans')
  })

  it('keeps a hidden chapter itself in the order', () => {
    const { previous, next } = chapterNeighbours('trees', (slug) => slug !== 'trees')

    expect(previous?.slug).toBe('map')
    expect(next?.slug).toBe('treecluster')
  })

  it('rejects an unknown chapter', () => {
    expect(() => chapterNeighbours('nope')).toThrow(/unknown chapter/)
  })
})

describe('firstChapter', () => {
  it('names the chapter the handbook opens with', () => {
    expect(firstChapter()).toEqual({
      slug: 'introduction',
      title: 'Über Green Ecolution',
      partTitle: 'Einstieg',
    })
  })
})

describe('firstChapter with a filter', () => {
  it('opens with the first visible chapter', () => {
    expect(firstChapter((slug) => slug !== 'introduction').slug).toBe('getting-started')
  })
})
