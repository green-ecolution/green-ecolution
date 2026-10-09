import type { ReactNode } from 'react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { render, screen, userEvent, within } from '@/test/utils'
import { UNRESTRICTED, type Permissions } from '@/lib/auth/permissions'
import type { PluginResponse } from '@/api/backendApi'

const permissions = vi.fn((): Permissions => UNRESTRICTED)
vi.mock('@/lib/auth/usePermissions', () => ({ usePermissions: () => permissions() }))

const updateMutate = vi.fn()
vi.mock('@/hooks/usePluginMutations', () => ({
  usePluginMutations: () => ({
    installPlugin: { mutate: vi.fn(), isPending: false },
    updatePlugin: { mutate: updateMutate, isPending: false },
    rotatePluginKey: { mutate: vi.fn(), isPending: false },
    uninstallPlugin: { mutate: vi.fn(), isPending: false },
  }),
}))

vi.mock('@tanstack/react-router', () => ({
  useNavigate: () => vi.fn(),
  Link: ({ children }: { children: ReactNode }) => <a href="#link">{children}</a>,
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
    permissions.mockReturnValue(UNRESTRICTED)
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
    await user.type(screen.getByLabelText(/^adresse/i), 'kataster.plugins.svc.cluster.local:80')
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
})
