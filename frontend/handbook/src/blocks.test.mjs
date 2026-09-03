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

  it('reads a standalone image as a numbered figure', () => {
    expect(parse('![Die Kartenübersicht](../images/map-overview.png)')).toEqual([
      { kind: 'figure', image: 'map-overview.png', caption: 'Die Kartenübersicht' },
    ])
  })

  it('rejects an image without a caption', () => {
    expect(() => parse('![](../images/map-overview.png)')).toThrow(/caption/)
  })

  it('rejects an image outside the handbook image folder', () => {
    expect(() => parse('![Karte](https://example.org/a.png)')).toThrow(/\.\.\/images\//)
  })

  it('rejects an image mixed into a text paragraph', () => {
    expect(() => parse('Siehe ![Karte](../images/map-overview.png) dort.')).toThrow(/own paragraph/)
  })

  it('rejects a non-PNG image', () => {
    expect(() => parse('![Diagramm](../images/diagram.svg)')).toThrow(/must be a PNG file/)
  })

  it('reads a gfm table', () => {
    const blocks = parse('| Status | Bedeutung |\n| --- | --- |\n| Aktiv | Läuft |')

    expect(blocks).toEqual([
      {
        kind: 'table',
        head: [[{ kind: 'text', value: 'Status' }], [{ kind: 'text', value: 'Bedeutung' }]],
        rows: [[[{ kind: 'text', value: 'Aktiv' }], [{ kind: 'text', value: 'Läuft' }]]],
      },
    ])
  })

  it('reads a fenced code block with its language', () => {
    expect(parse('```json\n{ "a": 1 }\n```')).toEqual([
      { kind: 'code', language: 'json', value: '{ "a": 1 }' },
    ])
  })
})
