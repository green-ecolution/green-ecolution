/// <reference types="node" />
import { describe, expect, it } from 'vitest'
import { createRequire } from 'node:module'
import fs from 'node:fs'
import path from 'node:path'
import { MODULE_BUILDS } from '../../modules.config'

const SHARED = [
  'react',
  'react-i18next',
  'i18next',
  '@tanstack/react-router',
  '@tanstack/react-query',
  'react-hook-form',
  'zustand',
  'maplibre-gl',
  'recharts',
  'vitest',
]

// A second copy of react-i18next or vitest would leave translations empty in tests.
// ESM-only packages (maplibre-gl) have no CommonJS entry, so compare the package directories.
const packageDir = (req: NodeJS.Require, name: string) => {
  const dir = req.resolve
    .paths(name)!
    .map((base) => path.join(base, name))
    .find((candidate) => fs.existsSync(candidate))
  return dir && fs.realpathSync(dir)
}

describe('module packages resolve shared libraries to the app instances', () => {
  const fromApp = createRequire(path.resolve(import.meta.dirname, '../../package.json'))
  const cases = MODULE_BUILDS.flatMap(({ key, root }) => {
    const fromModule = createRequire(path.join(root, 'package.json'))
    return SHARED.flatMap((name) => {
      const dir = packageDir(fromModule, name)
      // A library the module does not use cannot be duplicated.
      return dir ? [{ key, name, dir }] : []
    })
  })

  it.each(cases)('$key resolves $name to the app instance', ({ name, dir }) => {
    expect(dir).toBe(packageDir(fromApp, name))
  })
})
