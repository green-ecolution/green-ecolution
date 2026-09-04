import { mkdtemp, mkdir, writeFile, readFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { beforeEach, describe, expect, it } from 'vitest'
import { generate } from './generate.mjs'

let root

const chapter = (slug, part, body) =>
  [
    '---',
    `slug: ${slug}`,
    `title: Kapitel ${slug}`,
    `part: ${part}`,
    'summary: Kurzbeschreibung.',
    `routes: ['/${slug}']`,
    '---',
    '',
    body,
  ].join('\n')

beforeEach(async () => {
  root = await mkdtemp(join(tmpdir(), 'handbook-'))
  await mkdir(join(root, 'content', 'de'), { recursive: true })
  await writeFile(
    join(root, 'content', 'de', 'parts.json'),
    JSON.stringify([
      { id: 'intro', title: 'Einstieg' },
      { id: 'appendix', title: 'Anhang' },
    ]),
  )
})

describe('generate', () => {
  it('orders chapters by file prefix and groups them by part', async () => {
    await writeFile(
      join(root, 'content', 'de', '90-glossary.md'),
      chapter('glossary', 'appendix', '## A\n\nText.'),
    )
    await writeFile(
      join(root, 'content', 'de', '10-introduction.md'),
      chapter('introduction', 'intro', '## B\n\nText.'),
    )

    const { index } = await generate({ root, language: 'de' })

    expect(index.parts).toEqual([
      { id: 'intro', title: 'Einstieg', chapters: ['introduction'] },
      { id: 'appendix', title: 'Anhang', chapters: ['glossary'] },
    ])
    expect(Object.keys(index.chapters)).toEqual(['introduction', 'glossary'])
  })

  it('writes index, per-chapter content and the search index to disk', async () => {
    await writeFile(
      join(root, 'content', 'de', '10-introduction.md'),
      chapter('introduction', 'intro', '## B\n\nText.'),
    )

    await generate({ root, language: 'de' })

    const written = async (file) =>
      JSON.parse(await readFile(join(root, 'generated', file), 'utf8'))

    expect((await written('index.json')).chapters.introduction.title).toBe('Kapitel introduction')
    expect((await written('chapters/introduction.json')).blocks[0].kind).toBe('heading')
    expect((await written('search.json'))[0]).toMatchObject({ slug: 'introduction', anchor: 'b' })
  })

  it('rejects two chapters claiming the same slug', async () => {
    await writeFile(
      join(root, 'content', 'de', '10-a.md'),
      chapter('same', 'intro', '## A\n\nText.'),
    )
    await writeFile(
      join(root, 'content', 'de', '20-b.md'),
      chapter('same', 'intro', '## A\n\nText.'),
    )

    await expect(generate({ root, language: 'de' })).rejects.toThrow(/duplicate slug "same"/)
  })

  it('rejects a chapter referring to an unknown part', async () => {
    await writeFile(
      join(root, 'content', 'de', '10-a.md'),
      chapter('a', 'nowhere', '## A\n\nText.'),
    )

    await expect(generate({ root, language: 'de' })).rejects.toThrow(/unknown part "nowhere"/)
  })

  it('rejects a chapter reference pointing at a missing chapter', async () => {
    await writeFile(
      join(root, 'content', 'de', '10-a.md'),
      chapter('a', 'intro', '## A\n\nSiehe [dort](./missing.md).'),
    )

    await expect(generate({ root, language: 'de' })).rejects.toThrow(/unknown chapter "missing"/)
  })

  it('rejects a chapter reference inside a table cell pointing at a missing chapter', async () => {
    await writeFile(
      join(root, 'content', 'de', '10-a.md'),
      chapter(
        'a',
        'intro',
        '## A\n\n| Begriff | Siehe |\n| --- | --- |\n| Foo | [dort](./missing.md) |',
      ),
    )

    await expect(generate({ root, language: 'de' })).rejects.toThrow(/unknown chapter "missing"/)
  })

  it('accepts a chapter reference whose anchor matches a section of the target', async () => {
    await writeFile(
      join(root, 'content', 'de', '10-a.md'),
      chapter('a', 'intro', '## A\n\nSiehe [dort](./b.md#zweiter-abschnitt).'),
    )
    await writeFile(
      join(root, 'content', 'de', '20-b.md'),
      chapter('b', 'intro', '## Erster Abschnitt\n\nText.\n\n## Zweiter Abschnitt\n\nText.'),
    )

    await expect(generate({ root, language: 'de' })).resolves.toBeDefined()
  })

  it('rejects a chapter reference whose anchor is missing on the target', async () => {
    await writeFile(
      join(root, 'content', 'de', '10-a.md'),
      chapter('a', 'intro', '## A\n\nSiehe [dort](./b.md#nirgendwo).'),
    )
    await writeFile(
      join(root, 'content', 'de', '20-b.md'),
      chapter('b', 'intro', '## Erster Abschnitt\n\nText.'),
    )

    await expect(generate({ root, language: 'de' })).rejects.toThrow(
      /unknown anchor "nirgendwo" in chapter "b"/,
    )
  })

  it('accepts a chapter reference with no anchor even when the target has sections', async () => {
    await writeFile(
      join(root, 'content', 'de', '10-a.md'),
      chapter('a', 'intro', '## A\n\nSiehe [dort](./b.md).'),
    )
    await writeFile(
      join(root, 'content', 'de', '20-b.md'),
      chapter('b', 'intro', '## Erster Abschnitt\n\nText.'),
    )

    await expect(generate({ root, language: 'de' })).resolves.toBeDefined()
  })

  it('writes one typst file per chapter plus an ordered include list', async () => {
    await writeFile(
      join(root, 'content', 'de', '10-introduction.md'),
      chapter('introduction', 'intro', '## B\n\nText.'),
    )

    await generate({ root, language: 'de' })

    const read = (file) => readFile(join(root, 'generated', 'typst', file), 'utf8')

    await expect(read('introduction.typ')).resolves.toContain('#chapter("introduction"')
    await expect(read('chapters.typ')).resolves.toBe(
      '#import "../../typst/blocks.typ": *\n#part("intro", "Einstieg")\n#include "introduction.typ"\n#part("appendix", "Anhang")\n',
    )
  })
})
