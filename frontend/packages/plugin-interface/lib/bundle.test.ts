// @vitest-environment node
import { existsSync, readFileSync } from 'node:fs'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const bundle = fileURLToPath(new URL('../dist/plugin-interface.js', import.meta.url))

// Skipped rather than failed when the package has not been built: `pnpm test`
// alone does not build, while CI builds every package before running the
// suites, so the guard still runs where it matters.
describe.skipIf(!existsSync(bundle))('built bundle', () => {
  const code = () => readFileSync(bundle, 'utf8')

  it('keeps every react entry point external', () => {
    expect(code()).toMatch(/from\s*["']react\/jsx-runtime["']/)
  })

  // A bundled react subpath drags in react's CJS build, which reaches for
  // `require` and `process.env` and throws in the browser on the first import
  // of this package. Neither word belongs in a browser bundle at all, so both
  // are asserted as plain absences rather than as call shapes.
  it('reaches for no commonjs require', () => {
    expect(code()).not.toMatch(/\brequire\b/)
  })

  it('reads no process.env', () => {
    expect(code()).not.toMatch(/process\.env/)
  })
})
