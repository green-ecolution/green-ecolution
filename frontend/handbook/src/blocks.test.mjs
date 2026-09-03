import { describe, expect, it } from 'vitest'
import { unified } from 'unified'
import remarkParse from 'remark-parse'
import remarkGfm from 'remark-gfm'
import { toBlocks } from './blocks.mjs'

const parse = (markdown) => {
  const tree = unified().use(remarkParse).use(remarkGfm).parse(markdown)
  return toBlocks(tree.children, { file: 'demo.md', slug: 'demo' })
}

describe('toBlocks', () => {
  it('reads an unordered list as a plain list', () => {
    expect(parse('- Erstes\n- Zweites')).toEqual([
      {
        kind: 'list',
        items: [[{ kind: 'text', value: 'Erstes' }], [{ kind: 'text', value: 'Zweites' }]],
      },
    ])
  })

  it('reads an ordered list as steps', () => {
    expect(parse('1. Gruppe wählen\n2. Route berechnen')).toEqual([
      {
        kind: 'steps',
        items: [
          [{ kind: 'text', value: 'Gruppe wählen' }],
          [{ kind: 'text', value: 'Route berechnen' }],
        ],
      },
    ])
  })

  it('reads a github alert as a callout', () => {
    expect(parse('> [!WARNING]\n> Der Einsatz lässt sich danach nicht mehr ändern.')).toEqual([
      {
        kind: 'callout',
        tone: 'warning',
        children: [
          {
            kind: 'paragraph',
            children: [{ kind: 'text', value: 'Der Einsatz lässt sich danach nicht mehr ändern.' }],
          },
        ],
      },
    ])
  })

  it('allows a list inside a callout', () => {
    const blocks = parse(
      '> [!NOTE]\n> Voraussetzungen:\n>\n> - Rolle mit Schreibrecht\n> - Ein Fahrzeug',
    )

    expect(blocks[0].tone).toBe('note')
    expect(blocks[0].children.map((child) => child.kind)).toEqual(['paragraph', 'list'])
  })

  it('rejects a blockquote without an alert marker', () => {
    expect(() => parse('> Nur ein Zitat.')).toThrow(/alert marker/)
  })

  it('rejects an unknown alert tone', () => {
    expect(() => parse('> [!CAUTION]\n> Text.')).toThrow(/CAUTION/)
  })

  it('rejects a nested list', () => {
    expect(() => parse('- Erstes\n  - Verschachtelt')).toThrow(/nested/)
  })
})
