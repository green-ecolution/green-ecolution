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

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>
  </React.StrictMode>,
)
