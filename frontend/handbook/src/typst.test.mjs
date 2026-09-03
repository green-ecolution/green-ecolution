import { describe, expect, it } from 'vitest'
import { emitChapter, typstString } from './typst.mjs'

const meta = {
  slug: 'demo',
  title: 'Demo-Kapitel',
  part: 'intro',
  summary: 's',
  routes: [],
  sections: [],
}
const emit = (blocks) => emitChapter({ slug: 'demo', blocks }, meta)

describe('typstString', () => {
  it('escapes backslashes, quotes and newlines', () => {
    expect(typstString('a "b" \\ c\nd')).toBe('"a \\"b\\" \\\\ c\\nd"')
  })

  it('leaves typst markup characters untouched, because they are inside a string', () => {
    expect(typstString('#5 $x$ *a* [b]')).toBe('"#5 $x$ *a* [b]"')
  })
})

describe('emitChapter', () => {
  it('opens with the chapter call', () => {
    expect(emit([])).toContain('#chapter("demo", "Demo-Kapitel")')
  })

  it('emits headings as section calls', () => {
    expect(emit([{ kind: 'heading', level: 2, text: 'Board', anchor: 'board' }])).toContain(
      '#section(2, "board", "Board")',
    )
  })

  it('emits inline runs as nested calls', () => {
    const out = emit([
      {
        kind: 'paragraph',
        children: [
          { kind: 'text', value: 'Das Feld ' },
          { kind: 'strong', children: [{ kind: 'text', value: 'Art' }] },
          { kind: 'code', value: 'Tilia' },
        ],
      },
    ])

    expect(out).toContain('#para(txt("Das Feld "), strong(txt("Art")), tech("Tilia"))')
  })

  it('emits steps and bullets separately', () => {
    const items = [[{ kind: 'text', value: 'Eins' }], [{ kind: 'text', value: 'Zwei' }]]

    expect(emit([{ kind: 'steps', items }])).toContain(
      '#steps((para(txt("Eins")), para(txt("Zwei"))))',
    )
    expect(emit([{ kind: 'list', items }])).toContain(
      '#bullets((para(txt("Eins")), para(txt("Zwei"))))',
    )
  })

  it('emits a callout with a nested body', () => {
    const out = emit([
      {
        kind: 'callout',
        tone: 'warning',
        children: [{ kind: 'paragraph', children: [{ kind: 'text', value: 'Achtung.' }] }],
      },
    ])

    expect(out).toContain('#callout("warning")[\n#para(txt("Achtung."))\n]')
  })

  it('emits figures, tables and code blocks', () => {
    expect(emit([{ kind: 'figure', image: 'map.png', caption: 'Die Karte' }])).toContain(
      '#figure-image("map.png", txt("Die Karte"))',
    )
    expect(
      emit([
        {
          kind: 'table',
          head: [[{ kind: 'text', value: 'Status' }]],
          rows: [[[{ kind: 'text', value: 'Aktiv' }]]],
        },
      ]),
    ).toContain('#data-table((para(txt("Status")),), ((para(txt("Aktiv")),),))')
    expect(emit([{ kind: 'code', language: 'json', value: '{}' }])).toContain(
      '#code-block("json", "{}")',
    )
  })

  it('emits the three link kinds', () => {
    const link = (target) => ({
      kind: 'paragraph',
      children: [{ kind: 'link', target, children: [{ kind: 'text', value: 'hier' }] }],
    })

    expect(emit([link({ kind: 'external', href: 'https://a.de' })])).toContain(
      'link-external("https://a.de", txt("hier"))',
    )
    expect(emit([link({ kind: 'chapter', slug: 'trees', anchor: 'liste' })])).toContain(
      'xref-chapter("trees", "liste", txt("hier"))',
    )
    expect(emit([link({ kind: 'chapter', slug: 'trees' })])).toContain(
      'xref-chapter("trees", none, txt("hier"))',
    )
    expect(emit([link({ kind: 'app', to: '/map' })])).toContain('app-route("/map", txt("hier"))')
  })
})
