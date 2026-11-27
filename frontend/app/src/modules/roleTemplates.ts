import type { ComponentType } from 'react'
import { Eye, RadioTower, ShieldCheck } from 'lucide-react'
import type { I18nKey, ModuleManifest } from './types'

export interface RoleTemplateDisplay {
  nameKey: I18nKey
  icon: ComponentType<{ className?: string }>
}

const CORE_TEMPLATES: ReadonlyMap<string, RoleTemplateDisplay> = new Map([
  ['administrator', { nameKey: 'settings:roles.templateName.administrator', icon: ShieldCheck }],
  ['sensors', { nameKey: 'settings:roles.templateName.sensors', icon: RadioTower }],
  ['observer', { nameKey: 'settings:roles.templateName.observer', icon: Eye }],
])

/**
 * Name and icon of a delivered role, by the backend's `template_key`. The core
 * knows its own templates; a module declares the ones it ships. Pass every
 * registered manifest, not only the active ones: a role keeps its name where
 * the tenant has switched the module off.
 */
export function roleTemplateOf(
  key: string | null | undefined,
  manifests: readonly ModuleManifest[],
): RoleTemplateDisplay | undefined {
  if (key == null) return undefined
  return (
    CORE_TEMPLATES.get(key) ??
    manifests
      .flatMap(({ roleTemplates }) => roleTemplates ?? [])
      .find((template) => template.key === key)
  )
}
