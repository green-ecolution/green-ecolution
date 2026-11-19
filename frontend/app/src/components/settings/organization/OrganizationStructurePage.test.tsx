import type { ReactNode } from 'react'
import { describe, it, expect, vi, beforeAll, beforeEach } from 'vitest'
import { render, screen, userEvent, within } from '@/test/utils'
import type { Permissions } from '@/lib/auth/permissions'
import type { OrganizationResponse, UserResponse } from '@/api/backendApi'
import type { CreateOrganizationVariables } from '@/hooks/useOrganizationMutations'
import { getI18n } from '@/lib/i18n'
import type { ModuleManifest } from '@/modules/types'

const ORGS: OrganizationResponse[] = [
  { id: 'amt', name: 'Grünflächenamt', memberCount: 3, parentId: 'root', contactPersonId: 'u1' },
  { id: 'nord', name: 'Stadtgärtnerei Nord', memberCount: 4, parentId: 'amt' },
  { id: 'duburg', name: 'Team Duburg', memberCount: 6, parentId: 'nord' },
  { id: 'sued', name: 'Betriebshof Süd', memberCount: 2, parentId: 'amt', contactPersonId: 'u9' },
]

const person = { id: 'u1', firstName: 'Nils', lastName: 'Nissen' } as UserResponse

const context = vi.hoisted((): { orgId: string; perms: Permissions } => ({
  orgId: 'amt',
  perms: 'unrestricted',
}))

vi.mock('./useSettingsOrganization', () => ({
  useSettingsOrganization: () => ({
    orgId: context.orgId,
    organizations: ORGS,
    perms: context.perms,
    can: (required: readonly string[]) =>
      context.perms === 'unrestricted' ||
      required.length === 0 ||
      required.some((p) => (context.perms as Set<string>).has(p)),
  }),
}))

const probeManifest: ModuleManifest = {
  key: 'alpha',
  labelKey: 'probe:label' as ModuleManifest['labelKey'],
  i18n: { namespaces: [], load: () => Promise.resolve({}) },
}
vi.mock('@/modules/registry', () => ({ moduleManifests: [probeManifest] }))

const createOrganization = vi.fn()
vi.mock('@/hooks/useOrganizationMutations', () => ({
  useOrganizationMutations: () => ({
    createOrganization: {
      mutate: createOrganization,
      isPending: false,
      error: null,
      reset: vi.fn(),
    },
  }),
}))

const navigate = vi.fn()
vi.mock('@tanstack/react-router', () => ({
  Link: ({
    to,
    params,
    children,
    'aria-label': ariaLabel,
  }: {
    to: string
    params?: Record<string, string>
    children: ReactNode
    'aria-label'?: string
  }) => (
    <a
      href={Object.entries(params ?? {}).reduce(
        (path, [key, value]) => path.replace(`$${key}`, value),
        to,
      )}
      aria-label={ariaLabel}
    >
      {children}
    </a>
  ),
  useNavigate: () => navigate,
}))

vi.mock('@tanstack/react-query', async () => {
  const actual =
    await vi.importActual<typeof import('@tanstack/react-query')>('@tanstack/react-query')
  return {
    ...actual,
    useQuery: (options: { queryKey: unknown[] }) => {
      const [scope] = options.queryKey
      if (scope === 'modules') {
        return {
          data: {
            modules: ['alpha'],
            organizations: [{ organizationId: 'amt', active: ['alpha'] }],
          },
        }
      }
      if (scope === 'users') return { data: { data: [person] } }
      return { data: undefined }
    },
  }
})

const { default: OrganizationStructurePage } = await import('./OrganizationStructurePage')

const renderPage = () => render(<OrganizationStructurePage />)

describe('OrganizationStructurePage', () => {
  beforeAll(() => {
    getI18n().addResourceBundle('de', 'probe', { label: 'Probe' })
  })

  beforeEach(() => {
    vi.clearAllMocks()
    context.orgId = 'amt'
    context.perms = 'unrestricted'
    createOrganization.mockImplementation(
      (
        _vars: CreateOrganizationVariables,
        opts?: { onSuccess?: (org: { id: string }) => void },
      ) => {
        opts?.onSuccess?.({ id: 'west' })
      },
    )
  })

  it('lists every visible organization and marks the selected one', () => {
    renderPage()
    const rows = screen.getAllByRole('row').slice(1)
    expect(rows).toHaveLength(4)
    expect(within(rows[0]).getByText('Hier bist du gerade')).toBeInTheDocument()
    expect(rows[0]).toHaveAttribute('aria-current', 'true')
  })

  it('opens the overview of the clicked organization', () => {
    renderPage()
    expect(
      screen.getByRole('link', { name: 'Einstellungen von Stadtgärtnerei Nord öffnen' }),
    ).toHaveAttribute('href', '/settings/organization/nord')
  })

  it('shows total and direct members where they differ', () => {
    renderPage()
    expect(screen.getByText('15, davon 3 direkt')).toBeInTheDocument()
    expect(screen.getByText('6')).toBeInTheDocument()
  })

  it('names the contact person, or says that none is set', () => {
    renderPage()
    expect(screen.getByText('Nils Nissen')).toBeInTheDocument()
    expect(screen.getAllByText('Nicht hinterlegt')).toHaveLength(2)
  })

  it('does not claim that a contact person is missing when the name cannot be resolved', () => {
    renderPage()
    const row = screen.getByText('Betriebshof Süd').closest('tr')!
    expect(within(row).queryByText('Nicht hinterlegt')).not.toBeInTheDocument()
    expect(within(row).getByText('Name nicht verfügbar')).toHaveClass('sr-only')
    expect(within(row).getByText('–')).toBeInTheDocument()
  })

  it('shows the active modules by their label', () => {
    renderPage()
    expect(screen.getByText('Probe')).toBeInTheDocument()
  })

  it('offers creating a sub-organization only with organization:create', () => {
    context.perms = new Set(['organization:read'])
    renderPage()
    expect(
      screen.queryByRole('button', { name: 'Unterorganisation anlegen' }),
    ).not.toBeInTheDocument()
  })

  it('creates the sub-organization under the selected organization and opens it', async () => {
    renderPage()
    await userEvent.click(screen.getByRole('button', { name: 'Unterorganisation anlegen' }))
    await userEvent.type(screen.getByRole('textbox'), 'Betriebshof West')
    await userEvent.click(screen.getByRole('button', { name: 'Anlegen' }))
    expect(createOrganization).toHaveBeenCalledWith(
      { parentId: 'amt', name: 'Betriebshof West' },
      expect.anything(),
    )
    expect(navigate).toHaveBeenCalledWith({
      to: '/settings/organization/$orgId',
      params: { orgId: 'west' },
    })
  })
})
