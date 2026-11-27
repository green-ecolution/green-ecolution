import { existsSync, readdirSync } from 'node:fs'
import path from 'node:path'
import type { ModuleBuild } from './src/modules/buildConfig'

const ROUTES_DIRECTORY = 'src/routes'

/** A module package is a directory that holds `src/manifest.tsx`. */
export function discoverModules(modulesDir: string): ModuleBuild[] {
  return readdirSync(modulesDir, { withFileTypes: true })
    .filter((entry) => entry.isDirectory())
    .map((entry) => ({ key: entry.name, root: path.join(modulesDir, entry.name) }))
    .filter(({ root }) => existsSync(path.join(root, 'src/manifest.tsx')))
    .map((build) =>
      existsSync(path.join(build.root, ROUTES_DIRECTORY))
        ? { ...build, routesDirectory: ROUTES_DIRECTORY }
        : build,
    )
    .sort((a, b) => a.key.localeCompare(b.key))
}

export const MODULE_BUILDS: readonly ModuleBuild[] = discoverModules(
  path.resolve(import.meta.dirname, '../modules'),
)
