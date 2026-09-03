import React from 'react'
import ReactDOM from 'react-dom/client'
import './css/site.css'
import '@splidejs/react-splide/css'

import { createRouter, RouterProvider } from '@tanstack/react-router'
import { routeTree } from './routeTree.gen'
import NotFound from './components/layout/NotFound'
import ErrorFallback from './components/layout/ErrorFallback'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { getAuthSession } from '@/lib/auth/session'
import { AuthSessionProvider } from '@/lib/auth/AuthSessionProvider'
import { pendingLoading } from '@/lib/router'
import { startAnalytics } from '@/lib/analytics'
import { I18nextProvider } from 'react-i18next'
import { createI18n } from '@/lib/i18n'
import { UiTextBridge } from '@/lib/i18n/UiTextBridge'
import type { NavigationCrumbKey } from '@/lib/i18n/navigation'
import type { ParseKeys } from 'i18next'
import type { NAMESPACES } from '@/lib/i18n/languages'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 60_000,
      refetchOnWindowFocus: false,
      retry: 1,
    },
  },
})

const auth = getAuthSession()

const router = createRouter({
  routeTree,
  context: {
    queryClient,
    auth,
  },
  defaultErrorComponent: ({ error, reset }) => (
    <ErrorFallback error={error} resetErrorBoundary={reset} />
  ),
  defaultNotFoundComponent: () => <NotFound />,
  // A pending match without this renders null, so a loader that never settles
  // leaves a blank page behind. Only shows past defaultPendingMs (1s).
  defaultPendingComponent: pendingLoading({ key: 'common:state.loading' }),
  defaultPreload: 'intent',
  defaultPreloadStaleTime: 30_000,
  scrollRestoration: true,
})

startAnalytics(router)

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }

  /**
   * Static routes carry a bare navigation key resolved at render time; an
   * entity route carries a fully-qualified cross-namespace key plus its
   * interpolation params, resolved the same way so a language switch updates
   * it without a re-navigation. Only a crumb naming a loaded entity by its raw
   * data (no catalog can hold entity names) falls back to the literal text.
   */
  type Breadcrumb =
    | { titleKey: NavigationCrumbKey }
    | { titleKey: ParseKeys<typeof NAMESPACES>; params: Record<string, unknown> }
    | { title: string }

  interface StaticDataRouteOption {
    crumb?: Breadcrumb
  }
}

const isPWA =
  window.matchMedia('(display-mode: standalone)').matches ||
  (navigator as Navigator & { standalone?: boolean }).standalone === true

const removeLoader = () => {
  const loader = document.getElementById('app-loader')
  if (loader) {
    loader.classList.add('fade-out')
    setTimeout(() => loader.remove(), 300)
  }
}

if (!isPWA) {
  removeLoader()
} else {
  const SPLASH_MIN_DURATION = 1500
  const splashStart = Date.now()

  void new Promise<void>((r) => router.subscribe('onResolved', () => r())).then(() => {
    const elapsed = Date.now() - splashStart
    const remaining = Math.max(0, SPLASH_MIN_DURATION - elapsed)
    setTimeout(removeLoader, remaining)
  })
}

void createI18n()
  .then((i18n) => {
    ReactDOM.createRoot(document.getElementById('root')!).render(
      <React.StrictMode>
        <I18nextProvider i18n={i18n}>
          <UiTextBridge>
            <QueryClientProvider client={queryClient}>
              <AuthSessionProvider>
                <RouterProvider router={router} />
              </AuthSessionProvider>
            </QueryClientProvider>
          </UiTextBridge>
        </I18nextProvider>
      </React.StrictMode>,
    )
  })
  .catch((error: unknown) => {
    console.error('i18n bootstrap failed', error)
    removeLoader()
    // Untranslated on purpose: i18n itself is the component that failed to load,
    // so this path cannot depend on it (and must not mount I18nextProvider).
    ReactDOM.createRoot(document.getElementById('root')!).render(
      <div role="alert" className="mx-auto mt-[35vh] max-w-md px-4 text-center">
        <p className="mb-4">Die App konnte nicht geladen werden. Bitte lade die Seite neu.</p>
        <button
          type="button"
          onClick={() => window.location.reload()}
          className="rounded bg-primary px-4 py-2 text-white"
        >
          Neu laden
        </button>
      </div>,
    )
  })
