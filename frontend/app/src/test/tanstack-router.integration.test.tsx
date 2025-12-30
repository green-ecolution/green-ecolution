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
  Link,
  Outlet,
  useNavigate,
  useParams,
  useSearch,
  redirect,
  useBlocker,
  useRouterState,
} from '@tanstack/react-router'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { z } from 'zod'

function createTestQueryClient() {
  return new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: 0, staleTime: 0 },
      mutations: { retry: false },
    },
  })
}

describe('TanStack Router Integration Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    cleanup()
  })

  describe('Basic Routing', () => {
    it('renders root route correctly', async () => {
      const rootRoute = createRootRoute({
        component: () => <div data-testid="root">Root Content</div>,
      })

      const routeTree = rootRoute
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('root')).toBeInTheDocument()
      })

      expect(screen.getByText('Root Content')).toBeInTheDocument()
    })

    it('renders nested routes with Outlet', async () => {
      const rootRoute = createRootRoute({
        component: () => (
          <div data-testid="layout">
            <header>Header</header>
            <Outlet />
          </div>
        ),
      })

      const indexRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/',
        component: () => <main data-testid="index">Index Page</main>,
      })

      const routeTree = rootRoute.addChildren([indexRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('layout')).toBeInTheDocument()
      })

      expect(screen.getByText('Header')).toBeInTheDocument()
      expect(screen.getByTestId('index')).toBeInTheDocument()
    })

    it('renders correct route based on path', async () => {
      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const indexRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/',
        component: () => <div data-testid="home">Home</div>,
      })

      const aboutRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/about',
        component: () => <div data-testid="about">About</div>,
      })

      const routeTree = rootRoute.addChildren([indexRoute, aboutRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/about'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('about')).toBeInTheDocument()
      })
    })
  })

  describe('Link Navigation', () => {
    it('navigates when Link is clicked', async () => {
      const user = userEvent.setup()

      const rootRoute = createRootRoute({
        component: () => (
          <div>
            <nav>
              <Link to="/" data-testid="home-link">
                Home
              </Link>
              <Link to="/about" data-testid="about-link">
                About
              </Link>
            </nav>
            <Outlet />
          </div>
        ),
      })

      const indexRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/',
        component: () => <div data-testid="home-page">Home Page</div>,
      })

      const aboutRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/about',
        component: () => <div data-testid="about-page">About Page</div>,
      })

      const routeTree = rootRoute.addChildren([indexRoute, aboutRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('home-page')).toBeInTheDocument()
      })

      await user.click(screen.getByTestId('about-link'))

      await waitFor(() => {
        expect(screen.getByTestId('about-page')).toBeInTheDocument()
      })
    })

    it('applies active styles to current route Link', async () => {
      const rootRoute = createRootRoute({
        component: () => (
          <div>
            <Link to="/" activeProps={{ 'data-active': 'true' }} data-testid="home-link">
              Home
            </Link>
            <Outlet />
          </div>
        ),
      })

      const indexRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/',
        component: () => <div>Home</div>,
      })

      const routeTree = rootRoute.addChildren([indexRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('home-link')).toHaveAttribute('data-active', 'true')
      })
    })
  })

  describe('Route Params', () => {
    it('provides route params via useParams', async () => {
      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const userRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/user/$userId',
        component: function UserPage() {
          const { userId } = useParams({ from: '/user/$userId' })
          return <div data-testid="user-id">{userId}</div>
        },
      })

      const routeTree = rootRoute.addChildren([userRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/user/123'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('user-id')).toHaveTextContent('123')
      })
    })

    it('handles multiple route params', async () => {
      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const postRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/user/$userId/post/$postId',
        component: function PostPage() {
          const { userId, postId } = useParams({ from: '/user/$userId/post/$postId' })
          return (
            <div>
              <span data-testid="user-id">{userId}</span>
              <span data-testid="post-id">{postId}</span>
            </div>
          )
        },
      })

      const routeTree = rootRoute.addChildren([postRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/user/user-42/post/post-99'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('user-id')).toHaveTextContent('user-42')
        expect(screen.getByTestId('post-id')).toHaveTextContent('post-99')
      })
    })
  })

  describe('Search Params', () => {
    it('provides search params via useSearch', async () => {
      const searchSchema = z.object({
        page: z.number().catch(1),
        filter: z.string().optional(),
      })

      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const listRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/list',
        validateSearch: searchSchema,
        component: function ListPage() {
          const { page, filter } = useSearch({ from: '/list' })
          return (
            <div>
              <span data-testid="page">{page}</span>
              <span data-testid="filter">{filter ?? 'none'}</span>
            </div>
          )
        },
      })

      const routeTree = rootRoute.addChildren([listRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/list?page=5&filter=active'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('page')).toHaveTextContent('5')
        expect(screen.getByTestId('filter')).toHaveTextContent('active')
      })
    })

    it('uses default values for missing search params', async () => {
      const searchSchema = z.object({
        page: z.number().catch(1),
        limit: z.number().catch(10),
      })

      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const listRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/list',
        validateSearch: searchSchema,
        component: function ListPage() {
          const { page, limit } = useSearch({ from: '/list' })
          return (
            <div>
              <span data-testid="page">{page}</span>
              <span data-testid="limit">{limit}</span>
            </div>
          )
        },
      })

      const routeTree = rootRoute.addChildren([listRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/list'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('page')).toHaveTextContent('1')
        expect(screen.getByTestId('limit')).toHaveTextContent('10')
      })
    })
  })

  describe('Route Loaders', () => {
    it('executes loader before rendering component', async () => {
      const loaderFn = vi.fn().mockResolvedValue({ title: 'Loaded Data' })

      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const dataRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/data',
        loader: loaderFn,
        component: () => <div data-testid="data-page">Data Page</div>,
      })

      const routeTree = rootRoute.addChildren([dataRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/data'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('data-page')).toBeInTheDocument()
      })

      expect(loaderFn).toHaveBeenCalled()
    })

    it('provides loader data to component', async () => {
      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const dataRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/data',
        loader: () => ({ message: 'Hello from loader' }),
        component: function DataPage() {
          const data = dataRoute.useLoaderData()
          return <div data-testid="message">{data.message}</div>
        },
      })

      const routeTree = rootRoute.addChildren([dataRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/data'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('message')).toHaveTextContent('Hello from loader')
      })
    })

    it('loader receives route params', async () => {
      const loaderFn = vi.fn().mockImplementation(({ params }: { params: { userId: string } }) => ({
        userId: params.userId,
      }))

      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const userRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/user/$userId',
        loader: loaderFn,
        component: function UserPage() {
          const data = userRoute.useLoaderData()
          return <div data-testid="loaded-user">{data.userId}</div>
        },
      })

      const routeTree = rootRoute.addChildren([userRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/user/test-user'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('loaded-user')).toHaveTextContent('test-user')
      })

      expect(loaderFn).toHaveBeenCalledWith(
        expect.objectContaining({
          params: { userId: 'test-user' },
        }),
      )
    })

    it('loader receives router context', async () => {
      const queryClient = createTestQueryClient()

      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const dataRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/data',
        loader: ({ context }: { context: { queryClient: QueryClient } }) => ({
          hasQueryClient: context.queryClient instanceof QueryClient,
        }),
        component: function DataPage() {
          const data = dataRoute.useLoaderData()
          return <div data-testid="has-client">{data.hasQueryClient ? 'yes' : 'no'}</div>
        },
      })

      const routeTree = rootRoute.addChildren([dataRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/data'] }),
        context: { queryClient },
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('has-client')).toHaveTextContent('yes')
      })
    })
  })

  describe('useNavigate Hook', () => {
    it('navigates programmatically', async () => {
      const user = userEvent.setup()

      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const homeRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/',
        component: function HomePage() {
          const navigate = useNavigate()
          return (
            <button
              type="button"
              data-testid="nav-button"
              onClick={() => {
                void navigate({ to: '/destination' })
              }}
            >
              Go
            </button>
          )
        },
      })

      const destRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/destination',
        component: () => <div data-testid="destination">Destination</div>,
      })

      const routeTree = rootRoute.addChildren([homeRoute, destRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('nav-button')).toBeInTheDocument()
      })

      await user.click(screen.getByTestId('nav-button'))

      await waitFor(() => {
        expect(screen.getByTestId('destination')).toBeInTheDocument()
      })
    })

    it('navigates with search params', async () => {
      const user = userEvent.setup()

      const searchSchema = z.object({
        query: z.string().optional(),
      })

      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const homeRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/',
        component: function HomePage() {
          const navigate = useNavigate()
          return (
            <button
              type="button"
              data-testid="search-button"
              onClick={() => {
                void navigate({ to: '/search', search: { query: 'test' } })
              }}
            >
              Search
            </button>
          )
        },
      })

      const searchRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/search',
        validateSearch: searchSchema,
        component: function SearchPage() {
          const { query } = useSearch({ from: '/search' })
          return <div data-testid="search-query">{query}</div>
        },
      })

      const routeTree = rootRoute.addChildren([homeRoute, searchRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('search-button')).toBeInTheDocument()
      })

      await user.click(screen.getByTestId('search-button'))

      await waitFor(() => {
        expect(screen.getByTestId('search-query')).toHaveTextContent('test')
      })
    })

    it('navigates with route params', async () => {
      const user = userEvent.setup()

      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const homeRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/',
        component: function HomePage() {
          const navigate = useNavigate()
          return (
            <button
              type="button"
              data-testid="user-button"
              onClick={() => {
                void navigate({ to: '/user/$userId', params: { userId: 'abc-123' } })
              }}
            >
              Go to User
            </button>
          )
        },
      })

      const userRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/user/$userId',
        component: function UserPage() {
          const { userId } = useParams({ from: '/user/$userId' })
          return <div data-testid="user-param">{userId}</div>
        },
      })

      const routeTree = rootRoute.addChildren([homeRoute, userRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('user-button')).toBeInTheDocument()
      })

      await user.click(screen.getByTestId('user-button'))

      await waitFor(() => {
        expect(screen.getByTestId('user-param')).toHaveTextContent('abc-123')
      })
    })
  })

  describe('beforeLoad Guard', () => {
    it('executes beforeLoad before loader', async () => {
      const callOrder: string[] = []

      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const guardedRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/guarded',
        beforeLoad: () => {
          callOrder.push('beforeLoad')
        },
        loader: () => {
          callOrder.push('loader')
          return {}
        },
        component: () => <div data-testid="guarded">Guarded</div>,
      })

      const routeTree = rootRoute.addChildren([guardedRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/guarded'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('guarded')).toBeInTheDocument()
      })

      expect(callOrder).toEqual(['beforeLoad', 'loader'])
    })

    it('redirect in beforeLoad prevents route rendering', async () => {
      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const publicRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/public',
        component: () => <div data-testid="public">Public</div>,
      })

      const protectedRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/protected',
        beforeLoad: () => {
          throw redirect({ to: '/public' })
        },
        component: () => <div data-testid="protected">Protected</div>,
      })

      const routeTree = rootRoute.addChildren([publicRoute, protectedRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/protected'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('public')).toBeInTheDocument()
      })

      expect(screen.queryByTestId('protected')).not.toBeInTheDocument()
    })
  })

  describe('useBlocker Hook', () => {
    // Note: useBlocker with shouldBlockFn: () => true does not work reliably
    // with createMemoryHistory in tests. The blocking behavior is tested
    // in useFormNavigationBlocker.test.ts which mocks the hook directly.

    it('does not block when shouldBlockFn returns false', async () => {
      const user = userEvent.setup()

      function NonBlockingComponent() {
        useBlocker({
          shouldBlockFn: () => false,
          withResolver: true,
        })

        const navigate = useNavigate()

        return (
          <button
            type="button"
            data-testid="nav-away"
            onClick={() => {
              void navigate({ to: '/other' })
            }}
          >
            Leave
          </button>
        )
      }

      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const homeRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/',
        component: NonBlockingComponent,
      })

      const otherRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/other',
        component: () => <div data-testid="other">Other Page</div>,
      })

      const routeTree = rootRoute.addChildren([homeRoute, otherRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('nav-away')).toBeInTheDocument()
      })

      await user.click(screen.getByTestId('nav-away'))

      await waitFor(() => {
        expect(screen.getByTestId('other')).toBeInTheDocument()
      })
    })
  })

  describe('useRouterState Hook', () => {
    it('provides current router state', async () => {
      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const homeRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/',
        component: function HomePage() {
          const routerState = useRouterState()
          return <div data-testid="pathname">{routerState.location.pathname}</div>
        },
      })

      const routeTree = rootRoute.addChildren([homeRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('pathname')).toHaveTextContent('/')
      })
    })
  })

  describe('Not Found Handling', () => {
    it('renders defaultNotFoundComponent for unknown routes', async () => {
      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const homeRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/',
        component: () => <div>Home</div>,
      })

      const routeTree = rootRoute.addChildren([homeRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/unknown-path'] }),
        defaultNotFoundComponent: () => <div data-testid="not-found">404 Not Found</div>,
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('not-found')).toBeInTheDocument()
      })
    })
  })

  describe('Error Handling', () => {
    it('renders errorComponent on loader error', async () => {
      // Suppress expected React error boundary and router warning logging
      const noop = () => {
        /* intentionally empty */
      }
      const errorSpy = vi.spyOn(console, 'error').mockImplementation(noop)
      const warnSpy = vi.spyOn(console, 'warn').mockImplementation(noop)

      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const errorRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/error',
        loader: () => {
          throw new Error('Loader failed')
        },
        errorComponent: ({ error }) => <div data-testid="error">{error.message}</div>,
        component: () => <div>Should not render</div>,
      })

      const routeTree = rootRoute.addChildren([errorRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/error'] }),
      })

      render(<RouterProvider router={router} />)

      await waitFor(() => {
        expect(screen.getByTestId('error')).toHaveTextContent('Loader failed')
      })

      errorSpy.mockRestore()
      warnSpy.mockRestore()
    })
  })

  describe('Router with QueryClient Context', () => {
    it('provides queryClient through router context', async () => {
      const queryClient = createTestQueryClient()

      const rootRoute = createRootRoute({
        component: () => <Outlet />,
      })

      const dataRoute = createRoute({
        getParentRoute: () => rootRoute,
        path: '/data',
        loader: ({ context }: { context: { queryClient: QueryClient } }) => ({
          hasClient: context.queryClient instanceof QueryClient,
        }),
        component: function DataPage() {
          const data = dataRoute.useLoaderData()
          return <div data-testid="has-query-client">{data.hasClient ? 'yes' : 'no'}</div>
        },
      })

      const routeTree = rootRoute.addChildren([dataRoute])
      const router = createRouter({
        routeTree,
        history: createMemoryHistory({ initialEntries: ['/data'] }),
        context: { queryClient },
      })

      render(
        <QueryClientProvider client={queryClient}>
          <RouterProvider router={router} />
        </QueryClientProvider>,
      )

      await waitFor(() => {
        expect(screen.getByTestId('has-query-client')).toHaveTextContent('yes')
      })
    })
  })
})
