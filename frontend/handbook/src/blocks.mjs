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
        return [{ kind: 'paragraph', children: toInline(node.children, ctx) }]
      case 'list':
        return [{ kind: node.ordered ? 'steps' : 'list', items: listItems(node, ctx) }]
      case 'blockquote':
        return [callout(node, ctx)]
      default:
        return fail(ctx, `unsupported block node "${node.type}"`)
    }
  })
}
