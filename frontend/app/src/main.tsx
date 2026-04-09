import React from 'react'
import ReactDOM from 'react-dom/client'
import './css/site.css'
import 'leaflet/dist/leaflet.css'
import '@splidejs/react-splide/css'

import { createRouter, RouterProvider } from '@tanstack/react-router'
import { routeTree } from './routeTree.gen'
import NotFound from './components/layout/NotFound'
import ErrorFallback from './components/layout/ErrorFallback'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

const queryClient = new QueryClient()

const router = createRouter({
  routeTree,
  context: {
    queryClient,
  },
  defaultErrorComponent: ({ error, reset }) => (
    <ErrorFallback error={error} resetErrorBoundary={reset} />
  ),
  defaultNotFoundComponent: () => <NotFound />,
  defaultPreload: 'intent',
  defaultPreloadStaleTime: 0,
  scrollRestoration: true,
})

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }

  interface Breadcrumb {
    title: string
  }

  interface StaticDataRouteOption {
    crumb?: Breadcrumb
  }
}

const isPWA = window.matchMedia('(display-mode: standalone)').matches
    || (navigator as { standalone?: boolean }).standalone === true

const statusEl = document.querySelector('.app-loader-status')
const setStatus = (text: string) => {
  if (statusEl) statusEl.textContent = text
}

const removeLoader = () => {
  const loader = document.getElementById('app-loader')
  if (loader) {
    loader.classList.add('fade-out')
    setTimeout(() => loader.remove(), 300)
  }
}

const waitForBackend = async () => {
  const baseUrl = import.meta.env.VITE_BACKEND_BASEURL ?? '/api-local'
  const healthUrl = `${baseUrl}/v1/info`
  let attempts = 0

  while (true) {
    try {
      const res = await fetch(healthUrl, { method: 'GET' })
      if (res.ok) {
        setStatus('')
        return
      }
    } catch {
      // backend not reachable yet
    }

    attempts++
    if (attempts === 2) setStatus('Anwendung wird geladen\u2009\u2026')
    if (attempts === 6) setStatus('Laden dauert etwas l\u00e4nger als gew\u00f6hnlich\u2009\u2026')
    if (attempts === 15) setStatus('Keine Verbindung m\u00f6glich. Verbindungsversuch l\u00e4uft weiterhin\u2009\u2026')

    await new Promise((r) => setTimeout(r, 1000))
  }
}

if (!isPWA) {
  removeLoader()
} else {
  const SPLASH_MIN_DURATION = 1500
  const splashStart = Date.now()

  Promise.all([
    waitForBackend(),
    new Promise<void>((r) => router.subscribe('onResolved', () => r())),
  ]).then(() => {
    const elapsed = Date.now() - splashStart
    const remaining = Math.max(0, SPLASH_MIN_DURATION - elapsed)
    setTimeout(removeLoader, remaining)
  })
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </React.StrictMode>,
)
