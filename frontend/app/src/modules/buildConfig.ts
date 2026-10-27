/// <reference types="node" />
import path from 'node:path'
import { physical, type VirtualRouteNode } from '@tanstack/virtual-file-routes'

export interface ModuleBuildConfig {
  key: string
  routesDirectory: string
}

export interface ModuleBuild {
  root: string
  config: ModuleBuildConfig
}

export function moduleRouteNodes(
  routesDir: string,
  modules: readonly ModuleBuild[],
): VirtualRouteNode[] {
  return [
    physical('', '.'),
    ...modules.map(({ root, config }) =>
      physical('', path.relative(routesDir, path.join(root, config.routesDirectory))),
    ),
  ]
}
