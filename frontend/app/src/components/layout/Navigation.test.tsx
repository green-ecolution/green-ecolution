/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck - ad-hoc routes are not part of the generated route tree
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import {
  createRootRoute,
  createRoute,
  createRouter,
  createMemoryHistory,
  RouterProvider,
  Outlet,
} from '@tanstack/react-router'
import { UNRESTRICTED, type Permissions } from '@/lib/auth/permissions'

const permissions = vi.fn((): Permissions => new Set<string>())

vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: () => permissions(),
}))

vi.mock('@/lib/auth/authSessionContext', () => ({
  useAuthSession: () => ({ isAuthenticated: true, accessToken: null }),
}))

// The avatar hook would fire an unmocked /users/me request, which MSW rejects.
vi.mock('@/lib/auth/useCurrentUserAvatar', () => ({
  useCurrentUserAvatar: () => undefined,
}))

const { default: Navigation } = await import('./Navigation')

const renderNavigation = () => {
  const rootRoute = createRootRoute({ component: () => <Outlet /> })
  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => <Navigation isOpen closeSidebar={() => undefined} />,
  })
  const router = createRouter({
    routeTree: rootRoute.addChildren([indexRoute]),
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })

  return render(<RouterProvider router={router} />)
}

describe('Navigation', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('shows only the entries the permissions allow', async () => {
    permissions.mockReturnValue(new Set(['tree:read']))

    renderNavigation()

    await waitFor(() => {
      expect(screen.getByText('Bäume')).toBeInTheDocument()
    })
    // tree:read satisfies the map (tree OR cluster) and evaluations (any read).
    expect(screen.getByText('Karte')).toBeInTheDocument()
    expect(screen.getByText('Auswertung')).toBeInTheDocument()
    expect(screen.queryByText('Bewässerungsgruppen')).not.toBeInTheDocument()
    expect(screen.queryByText('Einsätze')).not.toBeInTheDocument()
    expect(screen.queryByText('Fahrzeuge')).not.toBeInTheDocument()
    expect(screen.queryByText('Mitarbeitende')).not.toBeInTheDocument()
    expect(screen.queryByText('Sensoren')).not.toBeInTheDocument()
  })

  it('drops the headline of a section without any reachable entry', async () => {
    permissions.mockReturnValue(new Set(['tree:read']))

    renderNavigation()

    await waitFor(() => {
      expect(screen.getByText('Grünflächen')).toBeInTheDocument()
    })
    expect(screen.queryByText('Einsatzplanung')).not.toBeInTheDocument()
  })

  it('shows every entry for unrestricted access', async () => {
    permissions.mockReturnValue(UNRESTRICTED)

    renderNavigation()

    await waitFor(() => {
      expect(screen.getByText('Mitarbeitende')).toBeInTheDocument()
    })
    expect(screen.getByText('Bewässerungsgruppen')).toBeInTheDocument()
    expect(screen.getByText('Sensoren')).toBeInTheDocument()
    expect(screen.getByText('Einsätze')).toBeInTheDocument()
  })

  it('keeps the always-open settings entry for a user without any grant', async () => {
    permissions.mockReturnValue(new Set<string>())

    renderNavigation()

    await waitFor(() => {
      expect(screen.getByText('Einstellungen')).toBeInTheDocument()
    })
    expect(screen.queryByText('Grünflächen')).not.toBeInTheDocument()
    expect(screen.queryByText('Bäume')).not.toBeInTheDocument()
  })
})
