import type { PluginViewResponse } from '@/api/backendApi'

/**
 * What the plugin viewer route renders for a given plugin. `external` mode
 * embeds the operator's own url, `proxied` mode the session url the backend
 * hands out on the plugin's own host. `proxied` without that url (the
 * instance has no proxy configured), `none` and a target on the app's own
 * origin each get a distinct message rather than one generic fallback.
 */
export type PluginViewKind =
  | { kind: 'iframe'; target: string }
  | { kind: 'proxied' }
  | { kind: 'unavailable' }
  | { kind: 'unsafeOrigin' }

const embeddable = (raw: string, appOrigin: string): PluginViewKind => {
  // The iframe carries allow-same-origin, which is only safe while the plugin
  // document sits on a foreign origin. The write paths reject the app's own
  // origin, but that check degrades to application.base_url alone under a CORS
  // wildcard, so the renderer refuses on its own rather than trusting the
  // stored string.
  let target: URL
  try {
    target = new URL(raw)
  } catch {
    return { kind: 'unavailable' }
  }
  if (target.origin === appOrigin) return { kind: 'unsafeOrigin' }
  return { kind: 'iframe', target: raw }
}

export const pluginViewKind = (
  plugin: Pick<PluginViewResponse, 'frontendMode' | 'frontendTarget' | 'frontendUrl'>,
  appOrigin: string = window.location.origin,
): PluginViewKind => {
  if (plugin.frontendMode === 'proxied') {
    return plugin.frontendUrl ? embeddable(plugin.frontendUrl, appOrigin) : { kind: 'proxied' }
  }
  if (plugin.frontendMode !== 'external' || !plugin.frontendTarget) return { kind: 'unavailable' }
  return embeddable(plugin.frontendTarget, appOrigin)
}

const IFRAME_FEATURES = ['camera', 'bluetooth'] as const

/** The iframe's `allow` attribute from the capabilities an operator granted. */
export const iframeAllow = (capabilities: readonly string[]): string =>
  IFRAME_FEATURES.filter((feature) => capabilities.includes(feature)).join('; ')
