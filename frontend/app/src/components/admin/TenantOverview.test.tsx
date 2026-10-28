import { Suspense } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { server } from '@/test/mocks/server'
import { DEMO_USER_ID, setOrganizationScope } from '@/lib/auth/scope/organizationScopeStore'

const navigate = vi.fn()
const showToast = vi.fn()
const switchScope = vi.fn()
vi.mock('@/hooks/createToast', () => ({ default: () => showToast }))
vi.mock('@tanstack/react-router', async () => {
  const actual =
    await vi.importActual<typeof import('@tanstack/react-router')>('@tanstack/react-router')
  return {
    ...actual,
    useNavigate: () => navigate,
    Link: ({ children, to }: { children: React.ReactNode; to: string }) => (
      <a href={to}>{children}</a>
    ),
  }
})
vi.mock('@/lib/auth/scope/useSwitchOrganizationScope', () => ({
  useSwitchOrganizationScope: () => switchScope,
}))

const { default: TenantOverview } = await import('./TenantOverview')

const ROOT = '01980000-0000-7000-8000-000000000001'
const TBZ = '01980000-0000-7000-8000-000000000002'

const respondWith = (organizations: unknown[]) =>
  server.use(
    http.get('*/v1/organizations', () => HttpResponse.json(organizations)),
    http.get('*/v1/organizations/resource-counts', () =>
      HttpResponse.json([
        { organization_id: TBZ, sensor_count: 7, vehicle_count: 2, plan_count: 11 },
      ]),
    ),
    http.get('*/v1/modules', () =>
      HttpResponse.json({
        modules: ['irrigation', 'waste'],
        organizations: [{ organization_id: TBZ, active: ['irrigation'] }],
      }),
    ),
  )

// The overview suspends on the organization list; the route provides the
// boundary in the app, the test provides its own.
const renderOverview = () => {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } })
  return render(
    <QueryClientProvider client={queryClient}>
      <Suspense fallback={null}>
        <TenantOverview />
      </Suspense>
    </QueryClientProvider>,
  )
}

describe('TenantOverview', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    setOrganizationScope(DEMO_USER_ID, null)
    switchScope.mockImplementation((id: string) => {
      setOrganizationScope(DEMO_USER_ID, id)
      return Promise.resolve()
    })
    // The generated client asks the auth session for a token; the demo session needs no OIDC settings.
    vi.stubEnv('VITE_AUTH_BYPASS', 'true')
  })

  it('lists the tenants with their counts and modules', async () => {
    respondWith([
      { id: ROOT, parent_id: null, name: 'Green Ecolution', member_count: 1 },
      { id: TBZ, parent_id: ROOT, name: 'TBZ', member_count: 5 },
    ])

    renderOverview()

    expect(await screen.findByText('TBZ')).toBeInTheDocument()
    expect(await screen.findByText('7')).toBeInTheDocument()
    expect(screen.getByText('2')).toBeInTheDocument()
    expect(screen.getByText('11')).toBeInTheDocument()
    expect(screen.getByText('5')).toBeInTheDocument()
    expect(await screen.findByText('Bewässerung')).toBeInTheDocument()
  })

  it('opens a tenant by setting the scope and going to the dashboard', async () => {
    respondWith([
      { id: ROOT, parent_id: null, name: 'Green Ecolution', member_count: 1 },
      { id: TBZ, parent_id: ROOT, name: 'TBZ', member_count: 5 },
    ])
    renderOverview()
    const user = userEvent.setup()

    await user.click(await screen.findByRole('button', { name: 'TBZ als Mandant öffnen' }))

    await waitFor(() => expect(switchScope).toHaveBeenCalledWith(TBZ))
    expect(navigate).toHaveBeenCalledWith({ to: '/dashboard' })
  })

  it('marks a tenant without active modules once the modules are loaded', async () => {
    const ANDERE = '01980000-0000-7000-8000-000000000003'
    respondWith([
      { id: ROOT, parent_id: null, name: 'Green Ecolution', member_count: 1 },
      { id: TBZ, parent_id: ROOT, name: 'TBZ', member_count: 5 },
      { id: ANDERE, parent_id: ROOT, name: 'Andere Stadt', member_count: 2 },
    ])

    renderOverview()

    expect(await screen.findByText('Keine Module aktiv')).toBeInTheDocument()
    expect(screen.getAllByText('Keine Module aktiv')).toHaveLength(1)
  })

  it('does not open the tenant and reports it when the scope switch fails', async () => {
    respondWith([
      { id: ROOT, parent_id: null, name: 'Green Ecolution', member_count: 1 },
      { id: TBZ, parent_id: ROOT, name: 'TBZ', member_count: 5 },
    ])
    switchScope.mockRejectedValueOnce(new Error('boom'))
    renderOverview()
    const user = userEvent.setup()

    await user.click(await screen.findByRole('button', { name: 'TBZ als Mandant öffnen' }))

    await waitFor(() =>
      expect(showToast).toHaveBeenCalledWith('Der Mandant konnte nicht geöffnet werden.', 'error'),
    )
    expect(navigate).not.toHaveBeenCalled()
    expect(screen.getByRole('button', { name: 'TBZ als Mandant öffnen' })).toBeEnabled()
  })

  it('does not navigate when the switch left the scope unset', async () => {
    respondWith([
      { id: ROOT, parent_id: null, name: 'Green Ecolution', member_count: 1 },
      { id: TBZ, parent_id: ROOT, name: 'TBZ', member_count: 5 },
    ])
    switchScope.mockResolvedValueOnce(undefined)
    renderOverview()
    const user = userEvent.setup()

    await user.click(await screen.findByRole('button', { name: 'TBZ als Mandant öffnen' }))

    await waitFor(() => expect(showToast).toHaveBeenCalled())
    expect(navigate).not.toHaveBeenCalled()
  })

  it('shows the loading label, not the empty state, while the list holds no root', async () => {
    respondWith([{ id: TBZ, parent_id: ROOT, name: 'TBZ', member_count: 5 }])

    renderOverview()

    expect(await screen.findByText('Mandanten werden geladen …')).toBeInTheDocument()
    expect(screen.queryByText('Diese Instanz hat noch keine Mandanten.')).not.toBeInTheDocument()
  })

  it('shows the empty state with a link to the organization page', async () => {
    respondWith([{ id: ROOT, parent_id: null, name: 'Green Ecolution', member_count: 1 }])

    renderOverview()

    expect(await screen.findByText('Diese Instanz hat noch keine Mandanten.')).toBeInTheDocument()
    expect(
      screen.getByRole('link', { name: 'Mandanten unter Organisation anlegen' }),
    ).toHaveAttribute('href', '/settings/organization')
  })
})
