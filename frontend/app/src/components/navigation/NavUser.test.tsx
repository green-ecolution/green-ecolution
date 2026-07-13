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
import NavUser from './NavUser'

// eslint-disable-next-line @typescript-eslint/no-empty-function
const noop = () => {}

const renderNavUser = (props: Partial<React.ComponentProps<typeof NavUser>> = {}) => {
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
    path: '/profile',
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

  return render(<RouterProvider router={router} />)
}

describe('NavUser', () => {
  beforeEach(() => {
    vi.clearAllMocks()
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
      expect(screen.getByRole('menuitem', { name: /ihr profil/i })).toBeInTheDocument()
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

  it('hides name and email on desktop widths when collapsed', async () => {
    renderNavUser({ collapsed: true })

    await waitFor(() => {
      expect(screen.getByText('Erika Mustermann')).toBeInTheDocument()
    })
    expect(screen.getByText('Erika Mustermann').parentElement).toHaveClass('lg:hidden')
  })
})
