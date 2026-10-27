import { moduleManifests } from '@/modules/registry'

/**
 * Where an `errors` key may be found, in lookup order. A module keeps the codes
 * only it raises under `<namespace>:errors.code.*` and may word a core code for
 * its own plans there. The module the failed request belonged to goes first,
 * then the core catalog, then every other module.
 */
export function errorCatalogKeys(key: string, module?: string): string[] {
  const core = `errors:${key}`
  if (!key.startsWith('code.')) return [core]
  const inModules = (keep: (moduleKey: string) => boolean) =>
    moduleManifests
      .filter(({ key: moduleKey }) => keep(moduleKey))
      .flatMap(({ i18n }) => i18n.namespaces.map((namespace) => `${namespace}:errors.${key}`))
  return [
    ...inModules((moduleKey) => moduleKey === module),
    core,
    ...inModules((moduleKey) => moduleKey !== module),
  ]
}
