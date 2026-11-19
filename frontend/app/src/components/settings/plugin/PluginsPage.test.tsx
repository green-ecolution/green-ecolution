import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { ReactNode } from 'react'
import type { OrganizationResponse, PluginResponse } from '@/api/backendApi'
import { render } from '@/test/utils'
import type { PluginInstallPayload } from './PluginInstallDialog'
import PluginsPage from './PluginsPage'

const ORGS: OrganizationResponse[] = [
  { id: 'root', name: 'Instanz', memberCount: 0 },
  { id: 'amt', name: 'Grünflächenamt', memberCount: 3, parentId: 'root' },
  { id: 'nord', name: 'Stadtgärtnerei Nord', memberCount: 4, parentId: 'amt' },
  { id: 'fremd', name: 'Fremdamt', memberCount: 1, parentId: 'root' },
]

const plugin = (slug: string, name: string, organizationId: string): PluginResponse =>
  ({
    id: slug,
    slug,
    name,
    organizationId,
    enabled: true,
    permissions: [],
    requiredPermissions: [],
    deviceCapabilities: [],
    frontend: { kind: 'none' },
  }) as unknown as PluginResponse

const state = vi.hoisted(
  (): {
    plugins: PluginResponse[]
    dialogProps: {
      organizationId: string
      onSubmit: (payload: PluginInstallPayload) => void
    } | null
  } => ({ plugins: [], dialogProps: null }),
)

vi.mock('../organization/useSettingsOrganization', () => ({
  useSettingsOrganization: () => ({
    orgId: 'amt',
    organizations: ORGS,
    perms: 'unrestricted',
    can: () => true,
  }),
}))

vi.mock('@/api/queries', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@/api/queries')>()),
  pluginsQuery: () => ({
    queryKey: ['test-plugins'],
    queryFn: () => Promise.resolve(state.plugins),
  }),
}))

vi.mock('@tanstack/react-router', async (importOriginal) => ({
  ...(await importOriginal<typeof import('@tanstack/react-router')>()),
  Link: ({
    children,
    to,
    params,
    ...rest
  }: {
    children: ReactNode
    to: string
    params: Record<string, string>
  }) => (
    <a {...rest} href={to.replace(/\$(\w+)/g, (_, key: string) => params[key] ?? '')}>
      {children}
    </a>
  ),
}))

const installPlugin = vi.fn()
vi.mock('@/hooks/usePluginMutations', () => ({
  usePluginMutations: () => ({ installPlugin: { mutate: installPlugin, isPending: false } }),
}))

vi.mock('./PluginInstallDialog', () => ({
  default: (props: {
    organizationId: string
    onSubmit: (payload: PluginInstallPayload) => void
  }) => {
    state.dialogProps = props
    return null
  },
}))

// Like the real dialog, the mock adds the organization it was given.
const submitInstall = (payload: Pick<PluginInstallPayload, 'slug' | 'name'>) =>
  state.dialogProps?.onSubmit({
    description: null,
    permissions: [],
    requiredPermissions: [],
    deviceCapabilities: [],
    frontend: { mode: 'none' },
    ...payload,
    organizationId: state.dialogProps.organizationId,
  } as unknown as PluginInstallPayload)

describe('PluginsPage', () => {
  beforeEach(() => {
    installPlugin.mockClear()
    state.plugins = [
      plugin('kataster-amt', 'Kataster Amt', 'amt'),
      plugin('kataster-nord', 'Kataster Nord', 'nord'),
      plugin('kataster-fremd', 'Kataster Fremd', 'fremd'),
    ]
  })

  it('lists the plugins of the selected organization and its sub-organizations', async () => {
    render(<PluginsPage />)
    expect(await screen.findByText('Kataster Amt')).toBeInTheDocument()
    expect(screen.getByText('Kataster Nord')).toBeInTheDocument()
    expect(screen.queryByText('Kataster Fremd')).not.toBeInTheDocument()
  })

  it('installs a plugin in the selected organization', async () => {
    render(<PluginsPage />)
    await userEvent.click(await screen.findByRole('button', { name: /Plugin installieren/ }))
    submitInstall({ slug: 'kataster', name: 'Kataster' })
    expect(installPlugin).toHaveBeenCalledWith(
      expect.objectContaining({ slug: 'kataster', organizationId: 'amt' }),
      expect.anything(),
    )
  })

  it('links a plugin to its detail under the selected organization', async () => {
    render(<PluginsPage />)
    expect(await screen.findByRole('link', { name: /Kataster Amt/ })).toHaveAttribute(
      'href',
      '/settings/organization/amt/plugins/kataster-amt',
    )
  })
})
