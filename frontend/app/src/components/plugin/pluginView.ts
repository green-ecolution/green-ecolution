import type { PluginViewResponse } from '@/api/backendApi'

/**
 * What the plugin viewer route renders for a given plugin. Only `external`
 * mode has a browser-reachable url today; `proxied`, `none` and a target on
 * the app's own origin each get a distinct message rather than being folded
 * into one generic fallback, since "in the cluster, not embeddable yet",
 * "this plugin has no view" and "this target is not safe to embed" are
 * different facts to report.
 */
export type PluginViewKind =
  | { kind: 'iframe'; target: string }
  | { kind: 'proxied' }
  | { kind: 'unavailable' }
  | { kind: 'unsafeOrigin' }

export const pluginViewKind = (
  plugin: Pick<PluginViewResponse, 'frontendMode' | 'frontendTarget'>,
  appOrigin: string = window.location.origin,
): PluginViewKind => {
  if (plugin.frontendMode === 'proxied') return { kind: 'proxied' }
  if (plugin.frontendMode !== 'external' || !plugin.frontendTarget) return { kind: 'unavailable' }

  // The iframe carries allow-same-origin, which is only safe while the plugin
  // document sits on a foreign origin. The write paths reject the app's own
  // origin, but that check degrades to application.base_url alone under a CORS
  // wildcard, so the renderer refuses on its own rather than trusting the
  // stored string.
  let target: URL
  try {
    target = new URL(plugin.frontendTarget)
  } catch {
    return { kind: 'unavailable' }
  }
  if (target.origin === appOrigin) return { kind: 'unsafeOrigin' }

  return { kind: 'iframe', target: plugin.frontendTarget }
}
