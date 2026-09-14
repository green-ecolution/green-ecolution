import type { TFunction } from 'i18next'

const SLUG_PATTERN = /^[a-z0-9]+(-[a-z0-9]+)*$/

/**
 * Mirrors `RESERVED_PLUGIN_SLUGS` in the Rust domain. `/plugins/me` is the
 * ingest self-lookup route and matches before the dynamic
 * `/plugins/{plugin_slug}`, so a plugin under this slug would be unreachable
 * by every admin endpoint afterwards. The backend rejects it either way; this
 * copy exists so the admin is told in the form rather than by a 400.
 */
export const RESERVED_SLUGS = ['me']

export const validateSlug = (value: string, t: TFunction<'settings'>): string | null => {
  if (value === '') return t('plugin.install.slugRequired')
  if (value.length > 64 || !SLUG_PATTERN.test(value)) return t('plugin.install.slugInvalid')
  if (RESERVED_SLUGS.includes(value)) return t('plugin.install.slugReserved', { slug: value })
  return null
}
