import { describe, expect, it } from 'vitest'
import { handbookIndex, loadChapter } from './index'
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
