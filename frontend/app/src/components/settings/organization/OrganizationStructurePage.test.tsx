import type { ReactNode } from 'react'
import { describe, it, expect, vi, beforeAll, beforeEach } from 'vitest'
import { cleanup, render, screen, userEvent, within } from '@/test/utils'
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

// Reachable but not readable: GET /organizations does not list it, so it has no member data.
const REACHABLE_ONLY: OrganizationResponse = {
  id: 'west',
  name: 'Betriebshof West',
  memberCount: 0,
  parentId: 'amt',
}

const context = vi.hoisted(
  (): { orgId: string; perms: Permissions; readable: string[] | null } => ({
    orgId: 'amt',
    perms: 'unrestricted',
    readable: null,
  }),
)

vi.mock('./useSettingsOrganization', () => ({
  useSettingsOrganization: () => ({
    orgId: context.orgId,
    organizations: [...ORGS, REACHABLE_ONLY],
    readableIds: new Set(context.readable ?? ORGS.map((org) => org.id)),
    perms: context.perms,
    can: (required: readonly string[]) =>
      context.perms === 'unrestricted' ||
      required.length === 0 ||
      required.some((p) => (context.perms as Set<string>).has(p)),
  }),
}))

const probeManifest: ModuleManifest = {
  key: 'alpha',
  order: 0,
  labelKey: 'probe:label' as ModuleManifest['labelKey'],
  icon: () => null,
  descriptionKey: 'probe:label' as ModuleManifest['labelKey'],
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
    context.readable = null
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

  it('lists no organization without member data', () => {
    renderPage()
    expect(screen.queryByText('Betriebshof West')).not.toBeInTheDocument()
  })

  it('opens the overview of the clicked organization', () => {
    renderPage()
    expect(screen.getByRole('link', { name: 'Stadtgärtnerei Nord' })).toHaveAttribute(
      'href',
      '/settings/organization/nord',
    )
  })

  it('tells assistive technology which organization is the current one', () => {
    renderPage()
    expect(
      screen.getByRole('link', { name: 'Grünflächenamt Hier bist du gerade' }),
    ).toBeInTheDocument()
  })

  it('says that a click on the name opens the settings', () => {
    renderPage()
    expect(
      screen.getByText(
        'Alle Organisationen, die du verwalten darfst. Ein Klick auf den Namen öffnet ihre Einstellungen.',
      ),
    ).toBeInTheDocument()
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

  describe('while the selected organization is the only one listed', () => {
    beforeEach(() => {
      context.readable = ['amt']
    })

    it('explains sub-organizations while the selected organization is the only one', () => {
      renderPage()
      expect(
        screen.getByRole('heading', { name: 'Grünflächenamt hat noch keine Unterorganisationen' }),
      ).toBeInTheDocument()
      expect(screen.queryByRole('table')).not.toBeInTheDocument()
      expect(
        screen.getByRole('img', {
          name: 'Beispiel: Grünflächenamt mit den Unterorganisationen Betriebshof und Beauftragte Firma',
        }),
      ).toBeInTheDocument()
      expect(screen.getByText('Eigenes Team')).toBeInTheDocument()
      expect(screen.getByText('Getrennte Daten')).toBeInTheDocument()
      expect(screen.getByText('Alles im Blick')).toBeInTheDocument()
      expect(screen.getByText(/^Gut zu wissen:/)).toBeInTheDocument()
    })

    it('leaves out the intro about clicking a name, since no name is listed', () => {
      renderPage()
      expect(screen.queryByText(/Ein Klick auf den Namen/)).not.toBeInTheDocument()
    })

    it('offers to create one only with the right', () => {
      renderPage()
      expect(screen.getAllByRole('button', { name: 'Unterorganisation anlegen' })).toHaveLength(1)
      cleanup()
      context.perms = new Set(['organization:read'])
      renderPage()
      expect(
        screen.queryByRole('button', { name: 'Unterorganisation anlegen' }),
      ).not.toBeInTheDocument()
    })

    it('opens the create dialog from the empty state', async () => {
      renderPage()
      await userEvent.click(screen.getByRole('button', { name: 'Unterorganisation anlegen' }))
      expect(screen.getByRole('dialog')).toBeInTheDocument()
    })
  })

  it('shows the table as soon as a second organization is visible', () => {
    context.readable = ['amt', 'nord']
    renderPage()
    expect(screen.getByRole('table')).toBeInTheDocument()
    expect(
      screen.queryByRole('heading', { name: /hat noch keine Unterorganisationen/ }),
    ).not.toBeInTheDocument()
    expect(screen.getAllByRole('button', { name: 'Unterorganisation anlegen' })).toHaveLength(1)
  })
})
