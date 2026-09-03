import { describe, expect, it } from 'vitest'
import { parseChapter } from './chapter.mjs'

const chapter = (body) =>
  [
    '---',
    'slug: demo',
    'title: Demo-Kapitel',
    'part: intro',
    'summary: Kurzbeschreibung.',
    "routes: ['/demo']",
    '---',
    '',
    body,
  ].join('\n')

describe('parseChapter', () => {
  it('reads the frontmatter into meta', () => {
    const { meta } = parseChapter(chapter('## Abschnitt\n\nText.'), { file: '10-demo.md' })

    expect(meta).toMatchObject({
      slug: 'demo',
      title: 'Demo-Kapitel',
      part: 'intro',
      summary: 'Kurzbeschreibung.',
      routes: ['/demo'],
    })
  })

  it('collects sections with github-style anchors', () => {
    const { meta } = parseChapter(
      chapter('## Route festlegen\n\nText.\n\n### Änderungen prüfen\n\nText.'),
      { file: '10-demo.md' },
    )

    expect(meta.sections).toEqual([
      { anchor: 'route-festlegen', title: 'Route festlegen', level: 2 },
      { anchor: 'anderungen-prufen', title: 'Änderungen prüfen', level: 3 },
    ])
  })

  it('flattens inline formatting in heading text and anchor', () => {
    const { blocks } = parseChapter(
      chapter('## Der Menüpunkt **Baumkataster** zeigt `Filter`\n\nText.'),
      { file: '10-demo.md' },
    )

    expect(blocks[0]).toEqual({
      kind: 'heading',
      level: 2,
      text: 'Der Menüpunkt Baumkataster zeigt Filter',
      anchor: 'der-menupunkt-baumkataster-zeigt-filter',
    })
  })

  it('turns paragraphs into inline runs', () => {
    const { blocks } = parseChapter(chapter('## A\n\nDas Feld **Art** nimmt `Tilia` an.'), {
      file: '10-demo.md',
    })

    expect(blocks[1]).toEqual({
      kind: 'paragraph',
      children: [
        { kind: 'text', value: 'Das Feld ' },
        { kind: 'strong', children: [{ kind: 'text', value: 'Art' }] },
        { kind: 'text', value: ' nimmt ' },
        { kind: 'code', value: 'Tilia' },
        { kind: 'text', value: ' an.' },
      ],
    })
  })

  it('collects section plain text for the search index', () => {
    const { search } = parseChapter(
      chapter('## Route festlegen\n\nErst Gruppen wählen, dann berechnen.'),
      { file: '10-demo.md' },
    )

    expect(search).toEqual([
      {
        slug: 'demo',
        anchor: 'route-festlegen',
        sectionTitle: 'Route festlegen',
        text: 'Erst Gruppen wählen, dann berechnen.',
      },
    ])
  })

  it('rejects a level-1 heading', () => {
    expect(() => parseChapter(chapter('# Titel\n\nText.'), { file: '10-demo.md' })).toThrow(
      /10-demo\.md.*level-1 heading/,
    )
  })

  it('rejects a missing frontmatter field', () => {
    const source = ['---', 'slug: demo', 'title: Demo', '---', '', '## A', '', 'Text.'].join('\n')

    expect(() => parseChapter(source, { file: '10-demo.md' })).toThrow(/part/)
  })
})
