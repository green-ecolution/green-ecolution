/* eslint-disable @typescript-eslint/ban-ts-comment */
// @ts-nocheck - Integration tests use ad-hoc routes not in generated route tree
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor, cleanup } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import {
  createRootRoute,
  createRoute,
  createRouter,
  createMemoryHistory,
  RouterProvider,
  Outlet,
} from '@tanstack/react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import NavUser from './NavUser'
import { userQueries } from '@/api/queries'
import { setOrganizationScope } from '@/lib/auth/scope/organizationScopeStore'
import { getI18n } from '@/lib/i18n'

// eslint-disable-next-line @typescript-eslint/no-empty-function
const noop = () => {}

const ORGS = [
  { id: 'root', name: 'Stadt', parentId: null },
  { id: 'firma_a', name: 'Firma A', parentId: 'root' },
]

const renderNavUser = (
  props: Partial<React.ComponentProps<typeof NavUser>> = {},
  organizations = ORGS,
) => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { staleTime: Infinity } } })
  queryClient.setQueryData(userQueries.myOrganizations().queryKey, organizations)
  const rootRoute = createRootRoute({
    component: () => <Outlet />,
  })

  const indexRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/',
    component: () => (
      <NavUser
        firstName="Erika"
        lastName="Mustermann"
        email="erika@example.com"
        collapsed={false}
        closeSidebar={noop}
        {...props}
      />
    ),
  })

  const profileRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/settings/profile',
    component: () => <div data-testid="profile-page">Profil</div>,
  })

  const logoutRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: '/logout',
    component: () => <div data-testid="logout-page">Logout</div>,
  })

  const routeTree = rootRoute.addChildren([indexRoute, profileRoute, logoutRoute])
  const router = createRouter({
    routeTree,
    history: createMemoryHistory({ initialEntries: ['/'] }),
  })

  return render(
    <QueryClientProvider client={queryClient}>
      <RouterProvider router={router} />
    </QueryClientProvider>,
  )
}

describe('NavUser', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    localStorage.clear()
    setOrganizationScope('u1', null)
  })

  afterEach(() => {
    cleanup()
  })

  it('shows initials, name and email in the trigger', async () => {
    renderNavUser()

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /benutzermenü/i })).toBeInTheDocument()
    })
    expect(screen.getByText('EM')).toBeInTheDocument()
    expect(screen.getByText('Erika Mustermann')).toBeInTheDocument()
    expect(screen.getByText('erika@example.com')).toBeInTheDocument()
  })

  it('opens the menu with profile and logout entries', async () => {
    const user = userEvent.setup()
    renderNavUser()

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /benutzermenü/i })).toBeInTheDocument()
    })
    await user.click(screen.getByRole('button', { name: /benutzermenü/i }))

    await waitFor(() => {
      expect(screen.getByRole('menuitem', { name: /dein profil/i })).toBeInTheDocument()
      expect(screen.getByRole('menuitem', { name: /abmelden/i })).toBeInTheDocument()
    })
  })

  it('navigates to logout and closes the sidebar when logging out', async () => {
    const user = userEvent.setup()
    const closeSidebar = vi.fn()
    renderNavUser({ closeSidebar })

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /benutzermenü/i })).toBeInTheDocument()
    })
    await user.click(screen.getByRole('button', { name: /benutzermenü/i }))
    await waitFor(() => {
      expect(screen.getByRole('menuitem', { name: /abmelden/i })).toBeInTheDocument()
    })
    await user.click(screen.getByRole('menuitem', { name: /abmelden/i }))

    await waitFor(() => {
      expect(screen.getByTestId('logout-page')).toBeInTheDocument()
    })
    expect(closeSidebar).toHaveBeenCalled()
  })

  it('switches the language from the menu without closing it', async () => {
    const user = userEvent.setup()
    renderNavUser()

    await waitFor(() => {
      expect(screen.getByRole('button', { name: /benutzermenü/i })).toBeInTheDocument()
    })
    await user.click(screen.getByRole('button', { name: /benutzermenü/i }))
    await waitFor(() => {
      expect(screen.getByRole('radiogroup', { name: 'Sprache' })).toBeInTheDocument()
    })

    await user.click(screen.getByRole('radio', { name: 'Englisch' }))

    expect(await screen.findByRole('radiogroup', { name: 'Language' })).toBeInTheDocument()
    expect(screen.getByRole('menuitem', { name: /log out/i })).toBeInTheDocument()

    await getI18n().changeLanguage('de')
  })

  it('hides name and email on desktop widths when collapsed', async () => {
    renderNavUser({ collapsed: true })

    await waitFor(() => {
      expect(screen.getByText('Erika Mustermann')).toBeInTheDocument()
    })
    expect(screen.getByText('Erika Mustermann').parentElement).toHaveClass('lg:hidden')
  })

  it('shows the active organization instead of the email when a scope is set', async () => {
    setOrganizationScope('u1', 'firma_a')
    renderNavUser()

    expect(await screen.findByText('Firma A')).toBeInTheDocument()
    expect(screen.queryByText('erika@example.com')).not.toBeInTheDocument()
  })

  it('keeps the email with a single reachable organization', async () => {
    setOrganizationScope('u1', 'firma_a')
    renderNavUser({}, [ORGS[1]])

    expect(await screen.findByText('erika@example.com')).toBeInTheDocument()
  })
})
