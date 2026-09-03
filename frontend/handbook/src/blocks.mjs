import { fail, plainText, slugify, toInline } from './inline.mjs'

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
      default:
        return fail(ctx, `unsupported block node "${node.type}"`)
    }
  })
}
