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
import { Settings, LogOut } from 'lucide-react'
import NavLink from './NavLink'

// eslint-disable-next-line @typescript-eslint/no-empty-function
const noop = () => {}

describe('NavLink', () => {
  const mockCloseSidebar = vi.fn()
  let consoleErrorSpy: ReturnType<typeof vi.spyOn<[typeof console, 'error']>>

  beforeEach(() => {
    vi.clearAllMocks()
    // Suppress React warnings about navIsOpen and closeSidebar props on DOM elements
    // These props are intentionally spread to the Link component and filtered internally
    consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(noop)
  })

  afterEach(() => {
    cleanup()
    // eslint-disable-next-line @typescript-eslint/no-unsafe-call, @typescript-eslint/no-unsafe-member-access
    consoleErrorSpy.mockRestore()
  })

  it('renders the NavLink with label and icon', async () => {
    const rootRoute = createRootRoute({
      component: () => <Outlet />,
    })

    const indexRoute = createRoute({
      getParentRoute: () => rootRoute,
      path: '/',
      component: () => (
        <ul>
          <NavLink
            label="Einstellungen"
            icon={<Settings data-testid="settings-icon" className="w-5 h-5" />}
            to="/settings"
            navIsOpen={true}
            closeSidebar={mockCloseSidebar}
          />
        </ul>
      ),
    })

    const settingsRoute = createRoute({
      getParentRoute: () => rootRoute,
      path: '/settings',
      component: () => <div data-testid="settings-page">Settings</div>,
    })

    const routeTree = rootRoute.addChildren([indexRoute, settingsRoute])
    const router = createRouter({
      routeTree,
      history: createMemoryHistory({ initialEntries: ['/'] }),
    })

    render(<RouterProvider router={router} />)

    await waitFor(() => {
      expect(screen.getByText('Einstellungen')).toBeInTheDocument()
      expect(screen.getByTestId('settings-icon')).toBeInTheDocument()
    })
  })

  it('navigates to the correct route when clicked', async () => {
    const user = userEvent.setup()

    const rootRoute = createRootRoute({
      component: () => <Outlet />,
    })

    const indexRoute = createRoute({
      getParentRoute: () => rootRoute,
      path: '/',
      component: () => (
        <ul>
          <NavLink
            label="Einstellungen"
            icon={<Settings className="w-5 h-5" />}
            to="/settings"
            navIsOpen={true}
            closeSidebar={mockCloseSidebar}
          />
        </ul>
      ),
    })

    const settingsRoute = createRoute({
      getParentRoute: () => rootRoute,
      path: '/settings',
      component: () => <div data-testid="settings-page">Settings</div>,
    })

    const routeTree = rootRoute.addChildren([indexRoute, settingsRoute])
    const router = createRouter({
      routeTree,
      history: createMemoryHistory({ initialEntries: ['/'] }),
    })

    render(<RouterProvider router={router} />)

    await waitFor(() => {
      expect(screen.getByText('Einstellungen')).toBeInTheDocument()
    })

    await user.click(screen.getByText('Einstellungen'))

    await waitFor(() => {
      expect(screen.getByTestId('settings-page')).toBeInTheDocument()
    })
  })

  it('calls closeSidebar when clicked', async () => {
    const user = userEvent.setup()

    const rootRoute = createRootRoute({
      component: () => <Outlet />,
    })

    const indexRoute = createRoute({
      getParentRoute: () => rootRoute,
      path: '/',
      component: () => (
        <ul>
          <NavLink
            label="Einstellungen"
            icon={<Settings className="w-5 h-5" />}
            to="/settings"
            navIsOpen={true}
            closeSidebar={mockCloseSidebar}
          />
        </ul>
      ),
    })

    const settingsRoute = createRoute({
      getParentRoute: () => rootRoute,
      path: '/settings',
      component: () => <div data-testid="settings-page">Settings</div>,
    })

    const routeTree = rootRoute.addChildren([indexRoute, settingsRoute])
    const router = createRouter({
      routeTree,
      history: createMemoryHistory({ initialEntries: ['/'] }),
    })

    render(<RouterProvider router={router} />)

    await waitFor(() => {
      expect(screen.getByText('Einstellungen')).toBeInTheDocument()
    })

    await user.click(screen.getByText('Einstellungen'))

    expect(mockCloseSidebar).toHaveBeenCalledTimes(1)
  })

  describe('Preload behavior', () => {
    it('does not trigger route preload when preload={false} and hovering over logout link', async () => {
      const user = userEvent.setup()
      const preloadFn = vi.fn()

      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const indexRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/',
        component: () => (
          <ul>
            <NavLink
              label="Ausloggen"
              icon={<Settings className="w-5 h-5" />}
              to="/logout"
              navIsOpen={true}
              closeSidebar={mockCloseSidebar}
              preload={false}
            />
          </ul>
        ),
      })

      const logoutRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/logout',
        loader: preloadFn,
        component: () => <div data-testid="logout-page">Logout</div>,
      })

      const routeTree = rootRoute.addChildren([indexRoute, logoutRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByText('Ausloggen')).toBeInTheDocument()
      })

      // Hover over the logout link
      await user.hover(screen.getByText('Ausloggen'))

      // Wait a bit to ensure no preload is triggered
      await new Promise((resolve) => setTimeout(resolve, 100))

      // The preload function should NOT have been called on hover
      expect(preloadFn).not.toHaveBeenCalled()
    })

    it('does not trigger route preload when preload={false} and hovering over login link', async () => {
      const user = userEvent.setup()
      const preloadFn = vi.fn()

      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const indexRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/',
        component: () => (
          <ul>
            <NavLink
              label="Anmelden"
              icon={<Settings className="w-5 h-5" />}
              to="/login"
              navIsOpen={true}
              closeSidebar={mockCloseSidebar}
              preload={false}
            />
          </ul>
        ),
      })

      const loginRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/login',
        loader: preloadFn,
        component: () => <div data-testid="login-page">Login</div>,
      })

      const routeTree = rootRoute.addChildren([indexRoute, loginRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByText('Anmelden')).toBeInTheDocument()
      })

      // Hover over the login link
      await user.hover(screen.getByText('Anmelden'))

      // Wait a bit to ensure no preload is triggered
      await new Promise((resolve) => setTimeout(resolve, 100))

      // The preload function should NOT have been called on hover
      expect(preloadFn).not.toHaveBeenCalled()
    })

    it('triggers route preload on hover when preload is not disabled (default behavior)', async () => {
      const user = userEvent.setup()
      const preloadFn = vi.fn()

      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const indexRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/',
        component: () => (
          <ul>
            <NavLink
              label="Einstellungen"
              icon={<Settings className="w-5 h-5" />}
              to="/settings"
              navIsOpen={true}
              closeSidebar={mockCloseSidebar}
              preload="intent"
            />
          </ul>
        ),
      })

      const settingsRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/settings',
        loader: preloadFn,
        component: () => <div data-testid="settings-page">Settings</div>,
      })

      const routeTree = rootRoute.addChildren([indexRoute, settingsRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByText('Einstellungen')).toBeInTheDocument()
      })

      // Hover over the settings link
      await user.hover(screen.getByText('Einstellungen'))

      // Wait for preload to be triggered
      await waitFor(
        () => {
          expect(preloadFn).toHaveBeenCalled()
        },
        { timeout: 1000 },
      )
    })

    it('still navigates correctly when preload={false}', async () => {
      const user = userEvent.setup()

      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const indexRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/',
        component: () => (
          <ul>
            <NavLink
              label="Ausloggen"
              icon={<LogOut className="w-5 h-5" />}
              to="/logout"
              navIsOpen={true}
              closeSidebar={mockCloseSidebar}
              preload={false}
            />
          </ul>
        ),
      })

      const logoutRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/logout',
        component: () => <div data-testid="logout-page">Logout</div>,
      })

      const routeTree = rootRoute.addChildren([indexRoute, logoutRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByText('Ausloggen')).toBeInTheDocument()
      })

      // Click the logout link
      await user.click(screen.getByText('Ausloggen'))

      // Should still navigate correctly
      await waitFor(() => {
        expect(screen.getByTestId('logout-page')).toBeInTheDocument()
      })
    })
  })

  describe('Label visibility', () => {
    it('shows label when navIsOpen is true', async () => {
      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const indexRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/',
        component: () => (
          <ul>
            <NavLink
              label="Einstellungen"
              icon={<Settings className="w-5 h-5" />}
              to="/settings"
              navIsOpen={true}
              closeSidebar={mockCloseSidebar}
            />
          </ul>
        ),
      })

      const settingsRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/settings',
        component: () => <div data-testid="settings-page">Settings</div>,
      })

      const routeTree = rootRoute.addChildren([indexRoute, settingsRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        const label = screen.getByText('Einstellungen')
        expect(label).toBeInTheDocument()
        expect(label).toHaveClass('lg:opacity-full', 'lg:block')
      })
    })

    it('hides label when navIsOpen is false', async () => {
      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const indexRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/',
        component: () => (
          <ul>
            <NavLink
              label="Einstellungen"
              icon={<Settings className="w-5 h-5" />}
              to="/settings"
              navIsOpen={false}
              closeSidebar={mockCloseSidebar}
            />
          </ul>
        ),
      })

      const settingsRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/settings',
        component: () => <div data-testid="settings-page">Settings</div>,
      })

      const routeTree = rootRoute.addChildren([indexRoute, settingsRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        const label = screen.getByText('Einstellungen')
        expect(label).toBeInTheDocument()
        expect(label).toHaveClass('lg:opacity-0', 'lg:hidden')
      })
    })
  })
})
