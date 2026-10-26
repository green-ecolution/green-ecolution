import { readFileSync } from 'node:fs'
import path from 'node:path'
import { describe, expect, it } from 'vitest'
import { MODULE_BUILDS } from '../../modules.config'

const SITE_CSS = path.resolve(import.meta.dirname, '../css/site.css')

function tailwindSources(): string[] {
  const css = readFileSync(SITE_CSS, 'utf8')
  return [...css.matchAll(/@source\s+["']([^"']+)["']/g)].map(([, glob]) =>
    path.resolve(path.dirname(SITE_CSS), glob),
  )
}

describe('tailwind sources', () => {
  it.each(MODULE_BUILDS.map((build) => [build.config.key, build.root]))(
    'scans the %s module package',
    (_key, root) => {
      const sample = path.join(root, 'src/components/Sample.tsx')
      expect(tailwindSources().some((glob) => path.matchesGlob(sample, glob))).toBe(true)
    },
  )
})
