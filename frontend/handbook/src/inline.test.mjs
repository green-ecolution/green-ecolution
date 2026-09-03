import { describe, expect, it } from 'vitest'
import { unified } from 'unified'
import remarkParse from 'remark-parse'
import remarkGfm from 'remark-gfm'
import { toInline } from './inline.mjs'

const parse = (markdown) => {
  const tree = unified().use(remarkParse).use(remarkGfm).parse(markdown)
  return toInline(tree.children[0].children, { file: 'demo.md', slug: 'demo' })
}

describe('link targets', () => {
  it('reads a chapter reference', () => {
    expect(parse('[Bewässerungsgruppen](./treecluster.md)')).toEqual([
      {
        kind: 'link',
        target: { kind: 'chapter', slug: 'treecluster' },
        children: [{ kind: 'text', value: 'Bewässerungsgruppen' }],
      },
    ])
  })

  it('reads a chapter reference with an anchor', () => {
    expect(parse('[Route](./watering-plans.md#route-festlegen)')[0].target).toEqual({
      kind: 'chapter',
      slug: 'watering-plans',
      anchor: 'route-festlegen',
    })
  })

  it('reads an app route reference', () => {
    expect(parse('[Karte öffnen](app:/map)')[0].target).toEqual({ kind: 'app', to: '/map' })
  })

  it('reads an external link', () => {
    expect(parse('[Projektseite](https://green-ecolution.de)')[0].target).toEqual({
      kind: 'external',
      href: 'https://green-ecolution.de',
    })
  })

  it('reads a mailto link', () => {
    expect(parse('[Kontakt](mailto:info@green-ecolution.de)')[0].target).toEqual({
      kind: 'external',
      href: 'mailto:info@green-ecolution.de',
    })
  })

  it('rejects a bare relative link', () => {
    expect(() => parse('[Irgendwas](../somewhere)')).toThrow(/unsupported link target/)
  })

  it('rejects raw html', () => {
    expect(() => parse('Text mit <kbd>Strg</kbd>.')).toThrow(/unsupported inline node "html"/)
  })
})
