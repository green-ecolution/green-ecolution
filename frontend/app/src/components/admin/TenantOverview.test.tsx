import { Suspense } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen, waitFor, within } from '@testing-library/react'
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

const permissionOverride = vi.hoisted((): { current: 'unrestricted' | Set<string> } => ({
  current: 'unrestricted',
}))
vi.mock('@/lib/auth/usePermissionsIn', () => ({
  usePermissionsIn: () => permissionOverride.current,
}))

interface Release {
  key: string
  released: boolean
  enabled: boolean
}
const DEFAULT_RELEASES: Release[] = [
  { key: 'irrigation', released: true, enabled: true },
  { key: 'waste', released: false, enabled: false },
]

const respondWith = (organizations: unknown[], releases: Release[] = DEFAULT_RELEASES) =>
  server.use(
    http.get('*/v1/organizations', () => HttpResponse.json(organizations)),
    http.get('*/v1/organizations/resource-counts', () =>
      HttpResponse.json([
        { organization_id: TBZ, sensor_count: 7, vehicle_count: 2, plan_count: 11 },
      ]),
    ),
    http.get('*/v1/modules/releases', () =>
      HttpResponse.json({
        modules: ['irrigation', 'waste'],
        tenants: [{ organization_id: TBZ, modules: releases }],
      }),
    ),
  )

const ORGANIZATIONS = [
  { id: ROOT, parent_id: null, name: 'Green Ecolution', member_count: 1 },
  { id: TBZ, parent_id: ROOT, name: 'TBZ', member_count: 5 },
]

const captureReleases = () => {
  const bodies: unknown[] = []
  server.use(
    http.put('*/v1/organizations/:id/module-releases', async ({ request, params }) => {
      bodies.push({ id: params.id, body: await request.json() })
      return HttpResponse.json({ organization_id: params.id, modules: [] })
    }),
  )
  return bodies
}

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
    permissionOverride.current = 'unrestricted'
    setOrganizationScope(DEMO_USER_ID, null)
    switchScope.mockImplementation((id: string) => {
      setOrganizationScope(DEMO_USER_ID, id)
      return Promise.resolve()
    })
    // The generated client asks the auth session for a token; the demo session needs no OIDC settings.
    vi.stubEnv('VITE_AUTH_BYPASS', 'true')
  })

  it('lists the tenants with their counts', async () => {
    respondWith([
      { id: ROOT, parent_id: null, name: 'Green Ecolution', member_count: 1 },
      { id: TBZ, parent_id: ROOT, name: 'TBZ', member_count: 5 },
    ])

    renderOverview()

    expect(await screen.findByText('TBZ')).toBeInTheDocument()
    expect(await screen.findByText('7 Sensoren, 2 Fahrzeuge, 11 Einsatzpläne')).toBeInTheDocument()
    expect(screen.getByText(/0 Unterorganisationen/)).toBeInTheDocument()
    expect(screen.getByText(/5 Mitarbeitende/)).toBeInTheDocument()
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

  describe('module release', () => {
    it('shows a column per module with its key and the release of each tenant', async () => {
      respondWith(ORGANIZATIONS)
      renderOverview()

      expect(
        await screen.findByRole('columnheader', { name: /Bewässerung\s*irrigation/ }),
      ).toBeInTheDocument()
      expect(screen.getByRole('columnheader', { name: /Abfall\s*waste/ })).toBeInTheDocument()
      const irrigation = screen.getByRole('switch', { name: 'Bewässerung für TBZ freigeben' })
      expect(irrigation).toHaveAttribute('aria-checked', 'true')
      const waste = screen.getByRole('switch', { name: 'Abfall für TBZ freigeben' })
      expect(waste).toHaveAttribute('aria-checked', 'false')
      expect(screen.getByText('Freigegeben')).toBeInTheDocument()
      expect(screen.getByText('Vom Mandanten eingeschaltet')).toBeInTheDocument()
      expect(screen.getByText('Gesperrt')).toBeInTheDocument()
      expect(screen.getByText('Für TBZ nicht sichtbar')).toBeInTheDocument()
    })

    it('releases a locked module at once', async () => {
      respondWith(ORGANIZATIONS)
      const bodies = captureReleases()
      renderOverview()
      const user = userEvent.setup()

      await user.click(await screen.findByRole('switch', { name: 'Abfall für TBZ freigeben' }))

      await waitFor(() => expect(bodies).toEqual([{ id: TBZ, body: { modules: { waste: true } } }]))
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument()
    })

    it('asks before locking a module the tenant has switched on', async () => {
      respondWith(ORGANIZATIONS)
      const bodies = captureReleases()
      renderOverview()
      const user = userEvent.setup()

      await user.click(await screen.findByRole('switch', { name: 'Bewässerung für TBZ freigeben' }))
      const dialog = await screen.findByRole('alertdialog')
      expect(within(dialog).getByText('Bewässerung für TBZ sperren?')).toBeInTheDocument()
      expect(bodies).toEqual([])

      await user.click(within(dialog).getByRole('button', { name: 'Abbrechen' }))
      await waitFor(() => expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument())
      expect(bodies).toEqual([])

      await user.click(screen.getByRole('switch', { name: 'Bewässerung für TBZ freigeben' }))
      await user.click(
        within(await screen.findByRole('alertdialog')).getByRole('button', { name: 'Sperren' }),
      )
      await waitFor(() =>
        expect(bodies).toEqual([{ id: TBZ, body: { modules: { irrigation: false } } }]),
      )
    })

    it('locks a released module the tenant has switched off without asking', async () => {
      respondWith(ORGANIZATIONS, [
        { key: 'irrigation', released: true, enabled: false },
        { key: 'waste', released: false, enabled: false },
      ])
      const bodies = captureReleases()
      renderOverview()
      const user = userEvent.setup()

      await user.click(await screen.findByRole('switch', { name: 'Bewässerung für TBZ freigeben' }))

      await waitFor(() =>
        expect(bodies).toEqual([{ id: TBZ, body: { modules: { irrigation: false } } }]),
      )
      expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument()
    })

    it('shows the state of a switched-off module', async () => {
      respondWith(ORGANIZATIONS, [
        { key: 'irrigation', released: true, enabled: false },
        { key: 'waste', released: false, enabled: false },
      ])
      renderOverview()

      expect(await screen.findByText('Vom Mandanten ausgeschaltet')).toBeInTheDocument()
    })

    it('disables the switches without the right to release', async () => {
      permissionOverride.current = new Set(['setting:read'])
      respondWith(ORGANIZATIONS)
      renderOverview()

      expect(
        await screen.findByRole('switch', { name: 'Bewässerung für TBZ freigeben' }),
      ).toBeDisabled()
      expect(screen.getByRole('switch', { name: 'Abfall für TBZ freigeben' })).toBeDisabled()
    })

    it('leaves the module columns out when the releases cannot be read', async () => {
      respondWith(ORGANIZATIONS)
      server.use(
        http.get('*/v1/modules/releases', () =>
          HttpResponse.json({ error: 'forbidden' }, { status: 403 }),
        ),
      )
      renderOverview()

      expect(await screen.findByText('TBZ')).toBeInTheDocument()
      expect(screen.getByRole('button', { name: 'TBZ als Mandant öffnen' })).toBeInTheDocument()
      await waitFor(() => expect(screen.queryByRole('switch')).not.toBeInTheDocument())
      expect(screen.queryByRole('columnheader', { name: /Bewässerung/ })).not.toBeInTheDocument()
      expect(showToast).not.toHaveBeenCalled()
    })

    it('reports a failed change and keeps the switch', async () => {
      respondWith(ORGANIZATIONS)
      server.use(
        http.put('*/v1/organizations/:id/module-releases', () =>
          HttpResponse.json({ error: 'boom' }, { status: 500 }),
        ),
      )
      renderOverview()
      const user = userEvent.setup()

      await user.click(await screen.findByRole('switch', { name: 'Abfall für TBZ freigeben' }))

      await waitFor(() =>
        expect(showToast).toHaveBeenCalledWith(
          'Die Freigabe konnte nicht geändert werden.',
          'error',
        ),
      )
      expect(screen.getByRole('switch', { name: 'Abfall für TBZ freigeben' })).toHaveAttribute(
        'aria-checked',
        'false',
      )
    })
  })
})
