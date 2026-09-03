export function readColors(css) {
  const root = /:root\s*\{([\s\S]*?)\}/.exec(css)
  if (!root) throw new Error('handbook: globals.css has no :root block')

  const tokens = {}
  const pattern = /--([\w-]+):\s*oklch\(([\d.]+)\s+([\d.]+)\s+([\d.]+)\)/g
  for (const match of root[1].matchAll(pattern)) {
    tokens[match[1]] = { l: Number(match[2]), c: Number(match[3]), h: Number(match[4]) }
  }
  return tokens
}

export function emitColors(tokens) {
  const entries = Object.entries(tokens).map(
    ([name, { l, c, h }]) => `  "${name}": oklch(${Number((l * 100).toFixed(4))}%, ${c}, ${h}deg),`,
  )
  return `#let colors = (\n${entries.join('\n')}\n)\n`
}
