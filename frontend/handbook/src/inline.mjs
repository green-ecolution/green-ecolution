export function slugify(text) {
  return text
    .toLowerCase()
    .replace(/ä/g, 'a')
    .replace(/ö/g, 'o')
    .replace(/ü/g, 'u')
    .replace(/ß/g, 'ss')
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')
}

export function fail(ctx, message) {
  throw new Error(`handbook: ${ctx.file}: ${message}`)
}

export function toInline(nodes, ctx) {
  return nodes.flatMap((node) => {
    switch (node.type) {
      case 'text':
        return [{ kind: 'text', value: node.value }]
      case 'strong':
        return [{ kind: 'strong', children: toInline(node.children, ctx) }]
      case 'emphasis':
        return [{ kind: 'emphasis', children: toInline(node.children, ctx) }]
      case 'inlineCode':
        return [{ kind: 'code', value: node.value }]
      default:
        return fail(ctx, `unsupported inline node "${node.type}"`)
    }
  })
}

export function plainText(inline) {
  return inline
    .map((run) => {
      switch (run.kind) {
        case 'text':
        case 'code':
          return run.value
        default:
          return plainText(run.children)
      }
    })
    .join('')
}
