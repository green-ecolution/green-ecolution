import { Suspense, type ReactNode } from 'react'
import { describe, expect, it, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { http, HttpResponse } from 'msw'
import { z } from 'zod'
import { server } from '@/test/mocks/server'
import type { ModuleManifest } from '@/modules/types'

// The real session needs OIDC settings the test environment does not have.
vi.mock('@/lib/auth/session', () => ({
  getAuthSession: () => ({ getAccessToken: () => Promise.resolve(null) }),
}))

vi.mock('@/modules/registry', () => {
  const probe = (key: string): ModuleManifest => ({
    key,
    labelKey: 'common:modules.irrigation',
    i18n: { namespaces: [], load: () => Promise.resolve({}) },
    map: {
      permissions: [],
      searchSchema: z.object({}),
      interactiveLayerIds: [],
      Overview: () => <p>{key} overview</p>,
      Toolbar: () => <p>{key} toolbar</p>,
      Overlay: () => <p>{key} overlay</p>,
    },
  })
  return { moduleManifests: [probe('alpha'), probe('beta')] }
})

const { MapOverlays, MapOverviews, MapToolbars } = await import('./MapModuleSlots')

const renderWithActive = (active: string[], ui: ReactNode) => {
  server.use(
    http.get('*/v1/modules', () =>
      HttpResponse.json({
        modules: ['alpha', 'beta'],
        organizations: [{ organizationId: 'org', active }],
      }),
    ),
  )
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  render(
    <QueryClientProvider client={queryClient}>
      <Suspense fallback={<p>loading</p>}>{ui}</Suspense>
    </QueryClientProvider>,
  )
}

describe('MapOverviews', () => {
  it('renders the overview of an active module and not that of an inactive one', async () => {
    renderWithActive(['alpha'], <MapOverviews />)

    expect(await screen.findByText('alpha overview')).toBeInTheDocument()
    expect(screen.queryByText('beta overview')).not.toBeInTheDocument()
  })

  it('renders no overview when no module is active', async () => {
    renderWithActive([], <MapOverviews />)

    await waitFor(() => expect(screen.queryByText('loading')).not.toBeInTheDocument())
    expect(screen.queryByText(/overview/)).not.toBeInTheDocument()
  })
})

describe('MapToolbars', () => {
  it('renders the toolbar of active modules only', async () => {
    renderWithActive(['beta'], <MapToolbars />)

    expect(await screen.findByText('beta toolbar')).toBeInTheDocument()
    expect(screen.queryByText('alpha toolbar')).not.toBeInTheDocument()
  })
})

describe('MapOverlays', () => {
  it('renders the overlay of active modules only', async () => {
    renderWithActive(['alpha', 'beta'], <MapOverlays />)

    expect(await screen.findByText('alpha overlay')).toBeInTheDocument()
    expect(screen.getByText('beta overlay')).toBeInTheDocument()
  })
})
