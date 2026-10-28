import { irrigationManifest } from '@green-ecolution/module-irrigation'
import { wasteManifest } from '@green-ecolution/module-waste'
import type { ModuleManifest } from './types'

export const moduleManifests: readonly ModuleManifest[] = [irrigationManifest, wasteManifest]
