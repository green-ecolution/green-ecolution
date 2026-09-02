import type { UiCatalog, UiTextKey, UiTranslate } from './catalog'

/**
 * The translator the package uses when no host application injects one, so a
 * component rendered in Storybook or a unit test still shows real copy.
 */
export function fallbackTranslate(catalog: UiCatalog): UiTranslate {
  return (key: UiTextKey, params?: Record<string, string | number>) => {
    const template = key
      .split('.')
      .reduce<unknown>(
        (node, segment) =>
          node !== null && typeof node === 'object'
            ? (node as Record<string, unknown>)[segment]
            : undefined,
        catalog,
      )
    if (typeof template !== 'string') return key
    if (!params) return template
    return Object.entries(params).reduce(
      (text, [name, value]) => text.split(`{${name}}`).join(String(value)),
      template,
    )
  }
}
