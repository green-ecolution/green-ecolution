import { describe, it, expect, vi } from 'vitest'
import { render, screen, waitFor } from '@/test/utils'
import {
  createRootRoute,
  createRoute,
  createRouter,
  createMemoryHistory,
  RouterProvider,
  Outlet,
} from '@tanstack/react-router'
import { z } from 'zod'
import { UNRESTRICTED } from '@/lib/auth/permissions'
import SettingsLayout from './SettingsLayout'

vi.mock('@/lib/auth/usePermissions', () => ({ usePermissions: () => UNRESTRICTED }))

const ownOrgId = vi.fn((): string | undefined => undefined)

vi.mock('@tanstack/react-query', async () => {
  const actual =
    await vi.importActual<typeof import('@tanstack/react-query')>('@tanstack/react-query')
  return {
    ...actual,
    useQuery: (options: { queryKey: unknown[] }) => {
      const [scope, second] = options.queryKey
      if (scope === 'users' && second === 'me') {
        const id = ownOrgId()
        return {
          data: { id: 'u-anna', roles: [], organization: id ? { id } : undefined },
          isLoading: false,
        }
      }
      if (scope === 'info' && second === 'services') {
        return { data: { items: [] }, isLoading: false }
      }
      return { data: undefined, isLoading: false }
    },
  }
})

// A minimal stand-in for the real route tree: only the paths SETTINGS_NAV
// links to are needed to resolve hrefs and active-link matching.
function buildRouter(initialPath: string) {
  const rootRoute = createRootRoute({
    component: () => (
      <SettingsLayout>
        <Outlet />
      </SettingsLayout>
    ),
  })

  const leaf = (path: string) =>
    createRoute({ getParentRoute: () => rootRoute, path, component: () => null })

  const organizationRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/settings/organization',
    validateSearch: z.object({ org: z.string().optional() }),
    component: () => null,
  })

  const routeTree = rootRoute.addChildren([
    leaf('/settings/profile'),
    organizationRoute,
    leaf('/settings/notifications'),
    leaf('/settings/team'),
    leaf('/settings/plugin'),
  ])

  return createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: [initialPath] }),
  })
}

describe('SettingsLayout', () => {
  it('keeps the own-organization shortcut inactive until its id resolves, leaving the plain entry active', async () => {
    ownOrgId.mockReturnValue(undefined)
    const router = buildRouter('/settings/organization')

    render(<RouterProvider router={router} />)

    await waitFor(() => {
      expect(screen.getByRole('link', { name: 'Organisation' })).toBeInTheDocument()
    })

    expect(screen.getByRole('link', { name: 'Organisation' })).toHaveAttribute(
      'data-status',
      'active',
    )
    expect(screen.getByRole('link', { name: 'Meine Organisation' })).not.toHaveAttribute(
      'data-status',
      'active',
    )
  })

  it('activates only the own-organization shortcut once its `?org=` matches the resolved id', async () => {
    ownOrgId.mockReturnValue('amt')
    const router = buildRouter('/settings/organization?org=amt')

    render(<RouterProvider router={router} />)

    await waitFor(() => {
      expect(screen.getByRole('link', { name: 'Meine Organisation' })).toHaveAttribute(
        'data-status',
        'active',
      )
    })

    expect(screen.getByRole('link', { name: 'Organisation' })).not.toHaveAttribute(
      'data-status',
      'active',
    )
  })
})
