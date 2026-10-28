import path from 'node:path'
import irrigation from '../modules/irrigation/module.config'
import waste from '../modules/waste/module.config'
import type { ModuleBuild } from './src/modules/buildConfig'

export const MODULE_BUILDS: readonly ModuleBuild[] = [
  { root: path.resolve(import.meta.dirname, '../modules/irrigation'), config: irrigation },
  { root: path.resolve(import.meta.dirname, '../modules/waste'), config: waste },
]
