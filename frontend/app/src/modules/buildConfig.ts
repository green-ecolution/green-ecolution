/// <reference types="node" />
import path from 'node:path'
import { physical, type VirtualRouteNode } from '@tanstack/virtual-file-routes'

export interface ModuleBuild {
  /** The directory name under `frontend/modules`, which is also the module's key. */
  key: string
  root: string
  /** Relative to `root`; unset for a module that ships no routes. */
  routesDirectory?: string
}

export function moduleRouteNodes(
  routesDir: string,
  modules: readonly ModuleBuild[],
): VirtualRouteNode[] {
  return [
    physical('', '.'),
    ...modules.flatMap(({ root, routesDirectory }) =>
      routesDirectory
        ? [physical('', path.relative(routesDir, path.join(root, routesDirectory)))]
        : [],
    ),
  ]
}
