import type { TFunction } from 'i18next'
import type { PluginFrontendDto } from '@/api/backendApi'

export type FrontendMode = 'none' | 'external' | 'proxied'

/**
 * Mirrors the backend's own check (`PluginFrontendDto::into_domain` in
 * `http/v1/dto/plugin.rs`) so an invalid external URL is caught before the
 * request round-trips, not just reported back as a 400.
 */
export const validateTarget = (
  mode: FrontendMode,
  value: string,
  t: TFunction<'settings'>,
): string | null => {
  if (mode === 'none') return null
  const trimmed = value.trim()
  if (trimmed === '') return t('plugin.install.targetRequired')

  if (mode === 'external') {
    let url: URL
    try {
      url = new URL(trimmed)
    } catch {
      return t('plugin.install.targetInvalidUrl')
    }
    const isLocalhost = url.hostname === 'localhost'
    if (url.protocol !== 'https:' && !isLocalhost) {
      return t('plugin.install.targetRequiresHttps')
    }
    // The iframe sandbox's allow-same-origin flag is safe only because the
    // plugin's document sits on a foreign origin; the app's own origin would
    // give it unrestricted script access to the app's DOM and storage.
    if (url.origin === window.location.origin) {
      return t('plugin.install.targetMatchesAppOrigin')
    }
    return null
  }

  const match = /^([^\s:]+):(\d+)$/.exec(trimmed)
  if (!match) return t('plugin.install.targetInvalidHostPort')
  const port = Number(match[2])
  if (port < 1 || port > 65535) return t('plugin.install.targetInvalidPort')
  return null
}

/**
 * The modes an administrator may pick. `proxied` is missing on purpose: the
 * domain accepts it, but nothing serves such a view yet, so choosing it only
 * produces a plugin whose view is a placeholder. A plugin that already carries
 * the mode keeps it in the list, otherwise editing its name would silently
 * rewrite its frontend to something else.
 */
export const frontendModeOptions = (
  t: TFunction<'settings'>,
  stored?: string,
): { value: FrontendMode; label: string }[] => {
  const options: { value: FrontendMode; label: string }[] = [
    { value: 'none', label: t('plugin.install.frontendModeOption.none') },
    { value: 'external', label: t('plugin.install.frontendModeOption.external') },
  ]
  if (stored === 'proxied') {
    options.push({ value: 'proxied', label: t('plugin.install.frontendModeOption.proxied') })
  }
  return options
}

export const buildFrontendDto = (mode: FrontendMode, target: string): PluginFrontendDto => {
  if (mode === 'none') return { mode: 'none' }
  return { mode, target: target.trim() }
}
