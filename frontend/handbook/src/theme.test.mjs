import { readFile } from 'node:fs/promises'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const typstDir = join(dirname(fileURLToPath(import.meta.url)), '..', 'typst')
const read = (file) => readFile(join(typstDir, file), 'utf8')

const EMITTED = [
  'chapter(slug, title)',
  'part(id, title)',
  'section(level, anchor, title)',
  'para(..runs)',
  'txt(value)',
  'tech(value)',
  'bullets(items)',
  'steps(items)',
  'callout(tone, body)',
  'figure-image(file, caption)',
  'data-table(head, rows)',
  'code-block(language, value)',
  'link-external(href, body)',
  'xref-chapter(slug, anchor, body)',
  'app-route(to, body)',
]

const NO_HARDCODED_COLOR =
  /\b(rgb|luma|cmyk)\(|color\.(rgb|luma|cmyk|hsl|hsv|oklch|oklab)\(|#[0-9a-fA-F]{6}(?:[0-9a-fA-F]{2})?\b|oklch\(/

describe('typst theme', () => {
  it('defines every function the emitter calls', async () => {
    const source = `${await read('theme.typ')}\n${await read('blocks.typ')}`
    const missing = EMITTED.filter((signature) => !source.includes(`#let ${signature}`))

    expect(missing).toEqual([])
  })

  it('never hardcodes a color', async () => {
    const theme = await read('theme.typ')
    const blocks = await read('blocks.typ')
    const manual = await read('manual.typ').catch(() => '')

    for (const [file, source] of [
      ['theme.typ', theme],
      ['blocks.typ', blocks],
      ['manual.typ', manual],
    ]) {
      expect(source, file).not.toMatch(NO_HARDCODED_COLOR)
    }
  })
})
