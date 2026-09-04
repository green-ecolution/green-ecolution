import { readFile } from 'node:fs/promises'
import { dirname, join } from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'
import { emitColors, readColors } from './colors.mjs'

const globalsCss = join(
  dirname(fileURLToPath(import.meta.url)),
  '../../packages/ui/src/styles/globals.css',
)

describe('readColors', () => {
  it('reads oklch tokens from the :root block', () => {
    const tokens = readColors(':root {\n  --green-dark: oklch(0.524 0.094 139.5);\n}')

    expect(tokens['green-dark']).toEqual({ l: 0.524, c: 0.094, h: 139.5 })
  })

  it('ignores tokens that only alias another token', () => {
    const tokens = readColors(':root {\n  --primary: var(--green-dark);\n}')

    expect(tokens).toEqual({})
  })

  it('ignores the dark-mode block', () => {
    const css = ':root {\n  --a: oklch(0.5 0 0);\n}\n.dark {\n  --b: oklch(0.9 0 0);\n}'

    expect(Object.keys(readColors(css))).toEqual(['a'])
  })

  it('reads the real stylesheet', async () => {
    const tokens = readColors(await readFile(globalsCss, 'utf8'))

    // Shape, not value: the theme follows the stylesheet, so changing a brand
    // token is a designer's call and must not break a handbook test.
    expect(tokens['green-dark']).toEqual({
      l: expect.any(Number),
      c: expect.any(Number),
      h: expect.any(Number),
    })
    expect(tokens['light']).toBeDefined()
  })
})

describe('emitColors', () => {
  it('writes a typst dictionary using oklch', () => {
    expect(emitColors({ 'green-dark': { l: 0.524, c: 0.094, h: 139.5 } })).toBe(
      '#let colors = (\n  "green-dark": oklch(52.4%, 0.094, 139.5deg),\n)\n',
    )
  })
})
