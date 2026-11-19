import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen, userEvent, within } from '@/test/utils'
import { UNRESTRICTED, type Permissions } from '@/lib/auth/permissions'
import type { PluginResponse } from '@/api/backendApi'

const permissions = vi.fn((orgId: string | null): Permissions => (orgId ? UNRESTRICTED : new Set()))
vi.mock('@/lib/auth/usePermissionsIn', () => ({
  usePermissionsIn: (orgId: string | null) => permissions(orgId),
}))

const updateMutate = vi.fn()
const uninstallMutate = vi.fn((_slug: string, options: { onSuccess: () => void }) =>
  options.onSuccess(),
)
vi.mock('@/hooks/usePluginMutations', () => ({
  usePluginMutations: () => ({
    installPlugin: { mutate: vi.fn(), isPending: false },
    updatePlugin: { mutate: updateMutate, isPending: false },
    rotatePluginKey: { mutate: vi.fn(), isPending: false },
    uninstallPlugin: { mutate: uninstallMutate, isPending: false },
  }),
}))

// The plugin belongs to a sub-organization of the one selected in the path.
vi.mock('../organization/useSettingsOrganization', () => ({
  useSettingsOrganization: () => ({
    orgId: 'amt',
    organizations: [
      { id: 'amt', name: 'Grünflächenamt', memberCount: 0 },
      { id: 'org-1', name: 'Stadtgärtnerei Nord', memberCount: 0, parentId: 'amt' },
    ],
  }),
}))

const navigate = vi.fn()
vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => navigate,
  Link: ({
    children,
    to,
    params,
  }: {
    children: ReactNode
    to?: string
    params?: Record<string, string>
  }) => <a href={to?.replace('$orgId', params?.orgId ?? '') ?? '#link'}>{children}</a>,
}))

vi.mock('@tanstack/react-query', async () => {
  const actual =
    await vi.importActual<typeof import('@tanstack/react-query')>('@tanstack/react-query')
  return { ...actual, useQuery: () => ({ data: [], isLoading: false }) }
})

const { default: PluginDetailPage } = await import('./PluginDetailPage')

const plugin: PluginResponse = {
  id: 'p1',
  slug: 'tbz-baumkataster',
  name: 'TBZ Baumkataster',
  description: null,
  organizationId: 'org-1',
  permissions: ['tree:create', 'tree:delete'],
  requiredPermissions: ['tree:read'],
  deviceCapabilities: [],
  frontendMode: 'none',
  frontendTarget: null,
  enabled: true,
  hasCredential: true,
  lastSeenAt: null,
  createdAt: null,
}

const pluginWithView: PluginResponse = {
  ...plugin,
  frontendMode: 'external',
  frontendTarget: 'https://sensor-setup.example.org',
  deviceCapabilities: ['camera'],
}

const proxiedPlugin: PluginResponse = {
  ...plugin,
  frontendMode: 'proxied',
  frontendTarget: 'kataster.plugins.svc.cluster.local:8080',
}

const pluginMatrixOf = () => within(screen.getByRole('group', { name: /rechte des plugins/i }))

const accessMatrixOf = () =>
  within(screen.getByRole('group', { name: /zugriffsrechte für die ansicht/i }))

const save = async () => {
  const user = userEvent.setup()
  await user.click(screen.getByRole('button', { name: /speichern/i }))
}

describe('PluginDetailPage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    permissions.mockImplementation(() => UNRESTRICTED)
  })

  /**
   * PluginService::update runs require_superset only when the request carries
   * a permission set, deliberately, so that a rename does not demand every
   * right the plugin holds. Sending both sets unconditionally turned that back
   * into a blanket check: an admin with plugin:update but without tree:delete
   * could not even change the name.
   */
  it('sends neither permission set when only the name changed', async () => {
    const user = userEvent.setup()
    render(<PluginDetailPage plugin={plugin} />)

    await user.clear(screen.getByLabelText(/^name/i))
    await user.type(screen.getByLabelText(/^name/i), 'Kataster')
    await save()

    expect(updateMutate).toHaveBeenCalledTimes(1)
    const { change } = updateMutate.mock.calls[0][0] as { change: Record<string, unknown> }
    expect(change.name).toBe('Kataster')
    expect(change).not.toHaveProperty('permissions')
    expect(change).not.toHaveProperty('requiredPermissions')
  })

  it('sends the plugin permissions once they actually changed', async () => {
    const user = userEvent.setup()
    render(<PluginDetailPage plugin={plugin} />)

    await user.click(pluginMatrixOf().getByRole('checkbox', { name: /bäume bearbeiten/i }))
    await save()

    const { change } = updateMutate.mock.calls[0][0] as { change: Record<string, unknown> }
    expect(change.permissions).toEqual(
      expect.arrayContaining(['tree:create', 'tree:delete', 'tree:update']),
    )
  })

  it('sends the access permissions alone when only they changed', async () => {
    const user = userEvent.setup()
    render(<PluginDetailPage plugin={plugin} />)

    await user.click(accessMatrixOf().getByRole('checkbox', { name: /fahrzeuge lesen/i }))
    await save()

    const { change } = updateMutate.mock.calls[0][0] as { change: Record<string, unknown> }
    expect(change).not.toHaveProperty('permissions')
    expect(change.requiredPermissions).toEqual(
      expect.arrayContaining(['tree:read', 'vehicle:read']),
    )
  })

  it('sends the device capabilities once they changed', async () => {
    const user = userEvent.setup()
    render(<PluginDetailPage plugin={pluginWithView} />)

    await user.click(screen.getByRole('checkbox', { name: /bluetooth/i }))
    await save()

    const { change } = updateMutate.mock.calls[0][0] as { change: Record<string, unknown> }
    expect(change.deviceCapabilities).toEqual(['camera', 'bluetooth'])
  })

  it('leaves the device capabilities out when they did not change', async () => {
    const user = userEvent.setup()
    render(<PluginDetailPage plugin={pluginWithView} />)

    await user.clear(screen.getByLabelText(/^name/i))
    await user.type(screen.getByLabelText(/^name/i), 'Sensor-Einrichtung')
    await save()

    const { change } = updateMutate.mock.calls[0][0] as { change: Record<string, unknown> }
    expect(change).not.toHaveProperty('deviceCapabilities')
  })

  /**
   * PluginService::update checks a frontend against the proxy allowlist
   * whenever the request carries one. A stored target that has since left the
   * allowlist would then block every save, even a rename.
   */
  it('leaves the frontend out when mode and target did not change', async () => {
    const user = userEvent.setup()
    render(<PluginDetailPage plugin={proxiedPlugin} />)

    await user.clear(screen.getByLabelText(/^name/i))
    await user.type(screen.getByLabelText(/^name/i), 'Kataster')
    await save()

    const { change } = updateMutate.mock.calls[0][0] as { change: Record<string, unknown> }
    expect(change.name).toBe('Kataster')
    expect(change).not.toHaveProperty('frontend')
  })

  it('sends the frontend once the target changed', async () => {
    const user = userEvent.setup()
    render(<PluginDetailPage plugin={proxiedPlugin} />)

    await user.clear(screen.getByLabelText(/^adresse/i))
    // Pasting renders once; typing the 37 characters re-rendered the page per key and timed out under load.
    await user.paste('kataster.plugins.svc.cluster.local:80')
    await save()

    const { change } = updateMutate.mock.calls[0][0] as { change: Record<string, unknown> }
    expect(change.frontend).toEqual({
      mode: 'proxied',
      target: 'kataster.plugins.svc.cluster.local:80',
    })
  })

  it.each([
    ['external', pluginWithView],
    ['proxied', proxiedPlugin],
  ])('offers to open the view of a plugin whose frontend is %s', (_, withView) => {
    render(<PluginDetailPage plugin={withView} />)
    expect(screen.getByText(/ansicht öffnen/i)).toBeInTheDocument()
  })

  it('offers no view for a plugin without a frontend', () => {
    render(<PluginDetailPage plugin={plugin} />)
    expect(screen.queryByText(/ansicht öffnen/i)).not.toBeInTheDocument()
  })

  it('names the plugin organization', () => {
    render(<PluginDetailPage plugin={plugin} />)
    expect(screen.getByText('Stadtgärtnerei Nord')).toBeInTheDocument()
  })

  it('links back to the plugins of the selected organization', () => {
    render(<PluginDetailPage plugin={plugin} />)
    expect(screen.getAllByRole('link').map((link) => link.getAttribute('href'))).toContain(
      '/settings/organization/amt/plugins',
    )
  })

  it('returns to the plugins of the selected organization after uninstalling', async () => {
    const user = userEvent.setup()
    render(<PluginDetailPage plugin={plugin} />)
    await user.click(screen.getByRole('button', { name: /deinstallieren/i }))
    await user.click(
      within(await screen.findByRole('alertdialog')).getByRole('button', {
        name: /deinstallieren/i,
      }),
    )
    expect(navigate).toHaveBeenCalledWith({
      to: '/settings/organization/$orgId/plugins',
      params: { orgId: 'amt' },
    })
  })

  it('offers the write controls with the rights in the plugin organization', () => {
    permissions.mockImplementation((orgId) =>
      orgId === 'org-1' ? new Set(['plugin:read', 'plugin:update', 'plugin:delete']) : new Set(),
    )
    render(<PluginDetailPage plugin={plugin} />)
    expect(screen.getByRole('button', { name: /speichern/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /deinstallieren/i })).toBeInTheDocument()
    expect(screen.getByLabelText(/^name/i)).toBeEnabled()
  })

  it('offers no write controls when the rights are held only in another organization', () => {
    permissions.mockImplementation((orgId) =>
      orgId === 'org-1'
        ? new Set(['plugin:read'])
        : new Set(['plugin:read', 'plugin:update', 'plugin:delete']),
    )
    render(<PluginDetailPage plugin={plugin} />)
    expect(screen.queryByRole('button', { name: /speichern/i })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /deinstallieren/i })).not.toBeInTheDocument()
    expect(screen.queryByRole('switch')).not.toBeInTheDocument()
    expect(screen.getByLabelText(/^name/i)).toBeDisabled()
  })
})
