export function typstString(value) {
  return `"${value.replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\n/g, '\\n')}"`
}

function inline(run) {
  switch (run.kind) {
    case 'text':
      return `txt(${typstString(run.value)})`
    case 'code':
      return `tech(${typstString(run.value)})`
    case 'strong':
      return `strong(${run.children.map(inline).join(', ')})`
    case 'emphasis':
      return `emph(${run.children.map(inline).join(', ')})`
    case 'link':
      return link(run)
    default:
      throw new Error(`handbook: cannot emit inline run "${run.kind}"`)
  }
}

function link(run) {
  const body = run.children.map(inline).join(', ')
  switch (run.target.kind) {
    case 'external':
      return `link-external(${typstString(run.target.href)}, ${body})`
    case 'chapter': {
      const anchor = run.target.anchor ? typstString(run.target.anchor) : 'none'
      return `xref-chapter(${typstString(run.target.slug)}, ${anchor}, ${body})`
    }
    case 'app':
      return `app-route(${typstString(run.target.to)}, ${body})`
    default:
      throw new Error(`handbook: cannot emit link target "${run.target.kind}"`)
  }
}

const para = (children) => `para(${children.map(inline).join(', ')})`

const tuple = (entries) => `(${entries.join(', ')}${entries.length === 1 ? ',' : ''})`

function block(node, slug) {
  switch (node.kind) {
    case 'heading':
      // Anchors are unique per chapter, the PDF holds every chapter at once —
      // so the label a cross-reference links to has to carry the chapter.
      return `#section(${node.level}, ${typstString(`${slug}-${node.anchor}`)}, ${typstString(node.text)})`
    case 'paragraph':
      return `#${para(node.children)}`
    case 'list':
      return `#bullets(${tuple(node.items.map(para))})`
    case 'steps':
      return `#steps(${tuple(node.items.map(para))})`
    case 'callout':
      return `#callout(${typstString(node.tone)})[\n${node.children.map((child) => block(child, slug)).join('\n')}\n]`
    case 'figure':
      return `#figure-image(${typstString(node.image)}, ${inline({ kind: 'text', value: node.caption })})`
    case 'table':
      return `#data-table(${tuple(node.head.map(para))}, ${tuple(node.rows.map((row) => tuple(row.map(para))))})`
    case 'code':
      return `#code-block(${node.language ? typstString(node.language) : 'none'}, ${typstString(node.value)})`
    default:
      throw new Error(`handbook: cannot emit block "${node.kind}"`)
  }
}

export function emitChapter(content, meta) {
  const head = [
    '#import "../../typst/blocks.typ": *',
    '',
    `#chapter(${typstString(meta.slug)}, ${typstString(meta.title)})`,
    '',
  ]
  return `${[...head, ...content.blocks.map((node) => block(node, meta.slug))].join('\n')}\n`
}
