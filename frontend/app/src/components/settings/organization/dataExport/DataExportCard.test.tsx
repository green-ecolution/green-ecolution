import { describe, it, expect, vi, beforeEach } from 'vitest'
import { http, HttpResponse } from 'msw'
import { server } from '@/test/mocks/server'
import { render, screen, userEvent, waitFor, within } from '@/test/utils'
import { UNRESTRICTED, type Permissions } from '@/lib/auth/permissions'
import DataExportCard from './DataExportCard'

vi.mock('@/lib/auth/session', () => ({
  getAuthSession: () => ({ getAccessToken: () => Promise.resolve(null) }),
}))

const permissions = vi.hoisted((): { current: Permissions } => ({ current: 'unrestricted' }))
vi.mock('@/lib/auth/usePermissions', () => ({
  usePermissions: (): Permissions => permissions.current,
}))

const TARGET_URL = '*/v1/organizations/org-1/data-export'

const targetBody = (overrides: Record<string, unknown> = {}) => ({
  activated_at: '2026-10-01T08:00:00Z',
  auth: { kind: 'oauth2_client_credentials', has_secret: true, client_id: 'green-ecolution' },
  base_url: 'https://frost.stadt.de/FROST-Server/v1.1',
  enabled: true,
  kind: 'sensorthings',
  organization_id: 'org-1',
  status: {
    last_run_at: '2026-11-03T12:04:00',
    last_success_at: '2026-11-03T12:04:00',
    last_run_stats: { things_written: 18, observations_written: 342, skipped: 0 },
  },
  ...overrides,
})

const answerTarget = (body: Record<string, unknown>, status = 200) =>
  server.use(http.get(TARGET_URL, () => HttpResponse.json(body, { status })))

const card = async () => {
  const heading = await screen.findByRole('heading', { name: 'Datenplattform' })
  const section = heading.closest('section')
  if (!section) throw new Error('data export card not found')
  return within(section)
}

describe('DataExportCard', () => {
  beforeEach(() => {
    permissions.current = UNRESTRICTED
  })

  it('renders nothing when the export is disabled on the instance', async () => {
    let requested = false
    server.use(
      http.get(TARGET_URL, () => {
        requested = true
        return HttpResponse.json({ error: 'off' }, { status: 503 })
      }),
    )
    render(<DataExportCard organizationId="org-1" />)

    await waitFor(() => expect(requested).toBe(true))
    await waitFor(() =>
      expect(screen.queryByRole('status', { name: 'Datenplattform wird geladen' })).toBeNull(),
    )
    expect(screen.queryByRole('heading', { name: 'Datenplattform' })).toBeNull()
  })

  it('reports a failed load', async () => {
    answerTarget({ error: 'boom' }, 500)
    render(<DataExportCard organizationId="org-1" />)

    expect(
      await screen.findByText('Die Einstellungen zur Datenplattform konnten nicht geladen werden.'),
    ).toBeInTheDocument()
  })

  it('offers the set-up to administrators when nothing is configured', async () => {
    answerTarget({ error: 'missing' }, 404)
    render(<DataExportCard organizationId="org-1" />)

    const view = await card()
    expect(view.getByText(/Überträgt Bäume, Behälter/)).toBeInTheDocument()
    expect(view.getByRole('button', { name: 'Datenplattform einrichten' })).toBeInTheDocument()
    expect(
      view.queryByText('Für diese Organisation ist keine Datenplattform eingerichtet.'),
    ).toBeNull()
  })

  it('only states the absence without setting:update', async () => {
    permissions.current = new Set(['setting:read'])
    answerTarget({ error: 'missing' }, 404)
    render(<DataExportCard organizationId="org-1" />)

    const view = await card()
    expect(
      view.getByText('Für diese Organisation ist keine Datenplattform eingerichtet.'),
    ).toBeInTheDocument()
    expect(view.queryByRole('button')).toBeNull()
  })

  it('shows address, authentication, status and the last run of a configured target', async () => {
    answerTarget(targetBody())
    render(<DataExportCard organizationId="org-1" />)

    const view = await card()
    expect(view.getByText('https://frost.stadt.de/FROST-Server/v1.1')).toBeInTheDocument()
    expect(view.getByText('Anmeldung: OAuth2 · Client-ID green-ecolution')).toBeInTheDocument()
    expect(view.getByText('Aktiv')).toBeInTheDocument()
    expect(
      view.getByText('Letzter Lauf 03.11.2026 12:04 · 18 Objekte · 342 Messwerte'),
    ).toBeInTheDocument()
    expect(view.queryByText(/Letzter Fehler/)).toBeNull()
    expect(view.getByRole('button', { name: 'Verbindung testen' })).toBeInTheDocument()
    expect(view.getByRole('button', { name: 'Bearbeiten' })).toBeInTheDocument()
    expect(view.getByRole('button', { name: 'Entfernen' })).toBeInTheDocument()
  })

  it('names the basic user and a run that has not happened yet', async () => {
    answerTarget(
      targetBody({ auth: { kind: 'basic', has_secret: true, username: 'alice' }, status: {} }),
    )
    render(<DataExportCard organizationId="org-1" />)

    const view = await card()
    expect(
      view.getByText('Anmeldung: Benutzername und Passwort · Benutzer alice'),
    ).toBeInTheDocument()
    expect(view.getByText('Noch kein Lauf')).toBeInTheDocument()
  })

  it('shows a paused target', async () => {
    answerTarget(targetBody({ enabled: false }))
    render(<DataExportCard organizationId="org-1" />)

    expect((await card()).getByText('Pausiert')).toBeInTheDocument()
  })

  it('shows a failing target with its last error', async () => {
    answerTarget(
      targetBody({
        status: { last_run_at: '2026-11-03T12:04:00', last_error: 'HTTP 502 from server' },
      }),
    )
    render(<DataExportCard organizationId="org-1" />)

    const view = await card()
    expect(view.getByText('Fehler')).toBeInTheDocument()
    expect(view.getByText('Letzter Fehler: HTTP 502 from server')).toBeInTheDocument()
  })

  it('hides the actions without setting:update', async () => {
    permissions.current = new Set(['setting:read'])
    answerTarget(targetBody())
    render(<DataExportCard organizationId="org-1" />)

    const view = await card()
    expect(view.getByText('https://frost.stadt.de/FROST-Server/v1.1')).toBeInTheDocument()
    expect(view.queryByRole('button')).toBeNull()
  })

  it.each([
    [
      { outcome: 'ok', data_array_supported: true },
      'Verbindung steht. Der Server unterstützt gebündelte Messwerte.',
    ],
    [
      { outcome: 'ok', data_array_supported: false },
      'Verbindung steht, aber der Server unterstützt keine gebündelten Messwerte. Rohmesswerte können nicht übertragen werden.',
    ],
    [{ outcome: 'unreachable', data_array_supported: false }, 'Der Server ist nicht erreichbar.'],
  ])('shows the connection test outcome %o', async (response, text) => {
    answerTarget(targetBody())
    server.use(http.post(`${TARGET_URL}/test`, () => HttpResponse.json(response)))
    render(<DataExportCard organizationId="org-1" />)

    const view = await card()
    await userEvent.click(view.getByRole('button', { name: 'Verbindung testen' }))

    expect(await view.findByText(text)).toBeInTheDocument()
  })

  it('reports a connection test request that failed', async () => {
    answerTarget(targetBody())
    server.use(
      http.post(`${TARGET_URL}/test`, () =>
        HttpResponse.json({ error: 'forbidden' }, { status: 403 }),
      ),
    )
    render(<DataExportCard organizationId="org-1" />)

    const view = await card()
    await userEvent.click(view.getByRole('button', { name: 'Verbindung testen' }))

    expect(
      await view.findByText('Für diese Aktion fehlt dir die Berechtigung.'),
    ).toBeInTheDocument()
  })

  it('asks before removing and then sends the DELETE', async () => {
    answerTarget(targetBody())
    let deleted = false
    server.use(
      http.delete(TARGET_URL, () => {
        deleted = true
        return new HttpResponse(null, { status: 204 })
      }),
    )
    render(<DataExportCard organizationId="org-1" />)

    const view = await card()
    await userEvent.click(view.getByRole('button', { name: 'Entfernen' }))

    const confirm = await screen.findByRole('alertdialog', { name: 'Datenplattform entfernen?' })
    expect(
      within(confirm).getByText(
        'Der Export dieser Organisation stoppt. Auf der Datenplattform wird nichts gelöscht.',
      ),
    ).toBeInTheDocument()
    expect(deleted).toBe(false)

    await userEvent.click(within(confirm).getByRole('button', { name: 'Entfernen' }))

    await waitFor(() => expect(deleted).toBe(true))
  })

  it('opens the edit dialog with the stored values', async () => {
    answerTarget(targetBody())
    render(<DataExportCard organizationId="org-1" />)

    const view = await card()
    await userEvent.click(view.getByRole('button', { name: 'Bearbeiten' }))

    const dialog = await screen.findByRole('dialog', { name: 'Datenplattform bearbeiten' })
    expect(within(dialog).getByLabelText('Adresse der SensorThings API')).toHaveValue(
      'https://frost.stadt.de/FROST-Server/v1.1',
    )
  })
})
