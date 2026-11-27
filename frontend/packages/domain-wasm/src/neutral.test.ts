import { readdirSync, readFileSync } from 'node:fs'
import path from 'node:path'
import { expect, it } from 'vitest'

const SRC = import.meta.dirname
// The package passes every validator through and knows none by name, so a new
// module needs no line here. Its drafts and form types live in its own package.
const MODULE_TERMS = /irrigation|waste|tree|cluster|bin\b/i

const sources = readdirSync(SRC).filter(
  (entry) => entry.endsWith('.ts') && !entry.endsWith('.test.ts'),
)

it.each(sources)('%s names no module', (file) => {
  const lines = readFileSync(path.join(SRC, file), 'utf8').split('\n')
  expect(lines.filter((line) => MODULE_TERMS.test(line))).toEqual([])
})
