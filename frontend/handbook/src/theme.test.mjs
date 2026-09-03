import { readFile } from 'node:fs/promises'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const typstDir = join(dirname(fileURLToPath(import.meta.url)), '..', 'typst')
const read = (file) => readFile(join(typstDir, file), 'utf8')

const EMITTED = [
  'chapter',
  'part',
  'section',
  'para',
  'txt',
  'tech',
  'bullets',
  'steps',
  'callout',
  'figure-image',
  'data-table',
  'code-block',
  'link-external',
  'xref-chapter',
  'app-route',
]

describe('typst theme', () => {
  it('defines every function the emitter calls', async () => {
    const source = `${await read('theme.typ')}\n${await read('blocks.typ')}`
    const missing = EMITTED.filter((name) => !source.includes(`#let ${name}(`))

    expect(missing).toEqual([])
  })

  it('never hardcodes a color', async () => {
    for (const file of ['theme.typ', 'blocks.typ', 'manual.typ']) {
      const source = await read(file).catch(() => '')
      expect(source, file).not.toMatch(/\b(rgb|luma)\(|#[0-9a-fA-F]{6}\b/)
      expect(source, file).not.toMatch(/oklch\(/)
    }
  })
})
