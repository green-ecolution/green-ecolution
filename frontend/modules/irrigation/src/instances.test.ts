/// <reference types="node" />
import { describe, expect, it } from 'vitest'
import { createRequire } from 'node:module'
import path from 'node:path'

// A second copy of react-i18next or vitest would leave translations empty in tests.
describe('module package resolves shared libraries to the app instances', () => {
  const fromApp = createRequire(path.resolve(import.meta.dirname, '../../../app/package.json'))
  const fromModule = createRequire(path.resolve(import.meta.dirname, '../package.json'))
  it.each([
    'react',
    'react-i18next',
    'i18next',
    '@tanstack/react-router',
    '@tanstack/react-query',
    'vitest',
  ])('%s', (name) => {
    expect(fromModule.resolve(name)).toBe(fromApp.resolve(name))
  })
})
