import { readAnalyticsScriptUrl } from '@/lib/auth/runtimeConfig'

interface AnalyticsApi {
  pageView: () => void
  trackEvent: (name: string, properties?: Record<string, unknown>) => void
}

declare global {
  interface Window {
    a?: AnalyticsApi
  }
}

interface RouterLike {
  subscribe: (event: 'onResolved', listener: () => void) => () => void
}

/**
 * Loads the page-view counter and reports one view per resolved navigation.
 * No-op unless the backend handed us a script URL, which only the demo
 * instance does.
 *
 * The script reads its tenant id back off its own `src`, so it has to be a
 * real element rather than an imported module.
 */
export function startAnalytics(router: RouterLike): void {
  const src = readAnalyticsScriptUrl()
  if (!src) return

  // The script only defines `window.a`; it never reports the initial view by
  // itself. Since it loads deferred, the first navigation usually resolves
  // before it exists — hold that view and send it once it does. Collapsing
  // several held views into one is correct: the report reads the current URL
  // at call time, so only the last one would have been distinguishable.
  let heldView = false
  const reportView = () => {
    if (window.a) {
      window.a.pageView()
    } else {
      heldView = true
    }
  }

  const script = document.createElement('script')
  script.src = src
  script.defer = true
  script.addEventListener('load', () => {
    if (heldView) {
      heldView = false
      window.a?.pageView()
    }
  })
  document.head.append(script)

  router.subscribe('onResolved', reportView)
}
