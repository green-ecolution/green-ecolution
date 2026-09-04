import { fail, plainText, slugify, toInline } from './inline.mjs'

const TONES = { NOTE: 'note', TIP: 'tip', IMPORTANT: 'important', WARNING: 'warning' }

function listItems(node, ctx) {
  return node.children.map((item) => {
    if (item.children.length !== 1 || item.children[0].type !== 'paragraph') {
      fail(ctx, 'nested or multi-paragraph list items are not supported')
    }
    return toInline(item.children[0].children, ctx)
  })
}

function callout(node, ctx) {
  const first = node.children[0]
  const marker =
    first?.type === 'paragraph' && first.children[0]?.type === 'text'
      ? /^\[!([A-Z]+)\]\n?/.exec(first.children[0].value)
      : null
  if (!marker) fail(ctx, 'blockquotes need an alert marker such as "> [!NOTE]"')

  const tone = TONES[marker[1]]
  if (!tone) fail(ctx, `unknown alert tone "${marker[1]}", use NOTE, TIP, IMPORTANT or WARNING`)

  const head = { ...first, children: [...first.children] }
  head.children[0] = { ...head.children[0], value: first.children[0].value.slice(marker[0].length) }
  const body =
    head.children[0].value === '' && head.children.length === 1
      ? node.children.slice(1)
      : [head, ...node.children.slice(1)]

  return { kind: 'callout', tone, children: toBlocks(body, ctx) }
}

function figure(node, ctx) {
  if (node.children.length !== 1)
    fail(ctx, 'an image needs its own paragraph, without surrounding text')

  const image = node.children[0]
  const caption = image.alt?.trim() ?? ''
  if (!caption) fail(ctx, 'every image needs a caption in its alt text')

  const match = /^\.\.\/images\/([\w-]+\.png)$/.exec(image.url)
  if (!match)
    fail(ctx, `image "${image.url}" must be a PNG file in ../images/ and be referenced from there`)

  return { kind: 'figure', image: match[1], caption }
}

function table(node, ctx) {
  const [head, ...body] = node.children
  const cells = (row) => row.children.map((cell) => toInline(cell.children, ctx))
  return { kind: 'table', head: cells(head), rows: body.map(cells) }
}

export function toBlocks(nodes, ctx) {
  return nodes.flatMap((node) => {
    switch (node.type) {
      case 'heading': {
        if (node.depth === 1)
          fail(ctx, 'level-1 heading is not allowed, the title lives in the frontmatter')
        if (node.depth > 3) fail(ctx, `heading depth ${node.depth} is not allowed, use ## or ###`)
        const text = plainText(toInline(node.children, ctx))
        return [{ kind: 'heading', level: node.depth, text, anchor: slugify(text) }]
      }
      case 'paragraph':
        return node.children.some((child) => child.type === 'image')
          ? [figure(node, ctx)]
          : [{ kind: 'paragraph', children: toInline(node.children, ctx) }]
      case 'list':
        return [{ kind: node.ordered ? 'steps' : 'list', items: listItems(node, ctx) }]
      case 'blockquote':
        return [callout(node, ctx)]
      case 'table':
        return [table(node, ctx)]
      case 'code':
        return [{ kind: 'code', language: node.lang ?? null, value: node.value }]
      default:
        return fail(ctx, `unsupported block node "${node.type}"`)
    }
  })
}
