/// <reference types="node" />
import { describe, expect, it } from 'vitest'
import { createRequire } from 'node:module'
import fs from 'node:fs'
import path from 'node:path'

// A second copy of react-i18next or vitest would leave translations empty in tests.
// ESM-only packages (maplibre-gl) have no CommonJS entry, so compare the package directories.
const packageDir = (req: NodeJS.Require, name: string) => {
  const dir = req.resolve
    .paths(name)!
    .map((base) => path.join(base, name))
    .find((candidate) => fs.existsSync(candidate))
  return dir && fs.realpathSync(dir)
}

describe('module package resolves shared libraries to the app instances', () => {
  const fromApp = createRequire(path.resolve(import.meta.dirname, '../../../app/package.json'))
  const fromModule = createRequire(path.resolve(import.meta.dirname, '../package.json'))
  it.each([
    'react',
    'react-i18next',
    'i18next',
    '@tanstack/react-router',
    '@tanstack/react-query',
    'react-hook-form',
    'zustand',
    'maplibre-gl',
    'vitest',
  ])('%s', (name) => {
    expect(packageDir(fromModule, name)).toBe(packageDir(fromApp, name))
  })
})
