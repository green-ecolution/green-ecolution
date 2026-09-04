import { unified } from 'unified'
import remarkParse from 'remark-parse'
import remarkGfm from 'remark-gfm'
import remarkFrontmatter from 'remark-frontmatter'
import { parse as parseYaml } from 'yaml'
import { toBlocks } from './blocks.mjs'
import { fail, plainText } from './inline.mjs'

const REQUIRED = ['slug', 'title', 'part', 'summary', 'routes']

const processor = unified().use(remarkParse).use(remarkGfm).use(remarkFrontmatter, ['yaml'])

function readFrontmatter(tree, ctx) {
  const node = tree.children[0]
  if (node?.type !== 'yaml') fail(ctx, 'chapter must start with a yaml frontmatter block')

  const data = parseYaml(node.value) ?? {}
  const missing = REQUIRED.filter((key) => data[key] === undefined)
  if (missing.length > 0) fail(ctx, `frontmatter is missing ${missing.join(', ')}`)
  if (!Array.isArray(data.routes)) fail(ctx, 'frontmatter routes must be a list')

  return data
}

function searchText(block) {
  switch (block.kind) {
    case 'paragraph':
      return plainText(block.children)
    case 'list':
    case 'steps':
      return block.items.map(plainText).filter(Boolean).join(' ')
    case 'table':
      return [block.head, ...block.rows]
        .flatMap((row) => row.map(plainText))
        .filter(Boolean)
        .join(' ')
    case 'callout':
      return block.children.map(searchText).filter(Boolean).join(' ')
    default:
      return ''
  }
}

function sectionsAndSearch(blocks, slug, title) {
  const sections = []
  // A chapter may open with content before its first heading — a lead
  // paragraph, a permission callout, or the glossary's single table, which
  // has no headings at all. The empty anchor lands such a hit at the top of
  // the chapter; the bucket is dropped again when nothing precedes a heading.
  const search = [{ slug, anchor: '', sectionTitle: title, text: '' }]

  for (const block of blocks) {
    if (block.kind === 'heading') {
      sections.push({ anchor: block.anchor, title: block.text, level: block.level })
      search.push({ slug, anchor: block.anchor, sectionTitle: block.text, text: '' })
      continue
    }
    const current = search.at(-1)
    const text = searchText(block)
    if (text) current.text = current.text ? `${current.text} ${text}` : text
  }

  return { sections, search: search.filter((entry) => entry.anchor !== '' || entry.text !== '') }
}

export function parseChapter(source, { file }) {
  const ctx = { file, slug: '' }
  const tree = processor.parse(source)
  const data = readFrontmatter(tree, ctx)
  ctx.slug = data.slug

  const blocks = toBlocks(tree.children.slice(1), ctx)
  const { sections, search } = sectionsAndSearch(blocks, data.slug, data.title)

  return {
    meta: {
      slug: data.slug,
      title: data.title,
      part: data.part,
      summary: data.summary,
      routes: data.routes,
      sections,
    },
    blocks,
    search,
  }
}
