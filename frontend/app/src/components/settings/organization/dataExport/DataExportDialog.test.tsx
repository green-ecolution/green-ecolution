import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, userEvent, waitFor, within } from '@/test/utils'
import { ResponseError, type DataExportTargetResponse } from '@/api/backendApi'
import { toDataExportRequest, type DataExportFormValues } from '@/api/dataExport'
import DataExportDialog from './DataExportDialog'

const URL_NOT_ALLOWED =
  'Diese Adresse ist für den Datenexport auf dieser Instanz nicht freigegeben. Wende dich an den Betrieb.'

const basicTarget: DataExportTargetResponse = {
  activatedAt: new Date('2026-10-01T08:00:00Z'),
  auth: { kind: 'basic', hasSecret: true, username: 'alice' },
  baseUrl: 'https://frost.stadt.de/FROST-Server/v1.1',
  enabled: true,
  kind: 'sensorthings',
  organizationId: 'org-1',
  status: {},
}

const onSubmit = vi.fn((_values: DataExportFormValues): Promise<unknown> => Promise.resolve())
const onOpenChange = vi.fn()

const renderDialog = (target?: DataExportTargetResponse) => {
  render(<DataExportDialog open target={target} onOpenChange={onOpenChange} onSubmit={onSubmit} />)
  return within(screen.getByRole('dialog'))
}

const submittedValues = (): DataExportFormValues => {
  const [values] = onSubmit.mock.calls[0]
  return values
}

describe('DataExportDialog', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    onSubmit.mockImplementation(() => Promise.resolve())
  })

  it('titles itself by mode', () => {
    renderDialog()
    expect(screen.getByRole('dialog', { name: 'Datenplattform einrichten' })).toBeInTheDocument()
  })

  it('saves an unauthenticated target', async () => {
    const dialog = renderDialog()
    await userEvent.type(
      dialog.getByLabelText('Adresse der SensorThings API'),
      'http://localhost:8180/FROST-Server/v1.1',
    )
    await userEvent.click(dialog.getByRole('button', { name: 'Speichern' }))

    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1))
    expect(toDataExportRequest(submittedValues())).toEqual({
      enabled: true,
      baseUrl: 'http://localhost:8180/FROST-Server/v1.1',
      auth: { kind: 'none' },
    })
    await waitFor(() => expect(onOpenChange).toHaveBeenCalledWith(false))
  })

  it('requires a URL', async () => {
    const dialog = renderDialog()
    await userEvent.click(dialog.getByRole('button', { name: 'Speichern' }))

    expect(await dialog.findByText('Gib die Adresse der SensorThings API an.')).toBeInTheDocument()
    expect(onSubmit).not.toHaveBeenCalled()
  })

  it.each([
    'ftp://frost.stadt.de/v1.1',
    'https://frost.stadt.de/v1.1?token=1',
    'https://frost.stadt.de/v1.1#top',
    'https://user:pw@frost.stadt.de/v1.1',
    'frost.stadt.de/v1.1',
  ])('refuses the URL %s', async (url) => {
    const dialog = renderDialog()
    await userEvent.type(dialog.getByLabelText('Adresse der SensorThings API'), url)
    await userEvent.click(dialog.getByRole('button', { name: 'Speichern' }))

    expect(
      await dialog.findByText(
        'Gib eine vollständige http- oder https-Adresse ohne Zugangsdaten, Parameter oder Anker an.',
      ),
    ).toBeInTheDocument()
    expect(onSubmit).not.toHaveBeenCalled()
  })

  it('requires the username and the password for a new basic target', async () => {
    const dialog = renderDialog()
    await userEvent.type(
      dialog.getByLabelText('Adresse der SensorThings API'),
      'https://frost.stadt.de/v1.1',
    )
    await userEvent.click(dialog.getByRole('radio', { name: 'Benutzername und Passwort' }))
    await userEvent.click(dialog.getByRole('button', { name: 'Speichern' }))

    expect(dialog.getByLabelText('Benutzername')).toHaveAccessibleDescription(
      'Dieses Feld ist erforderlich.',
    )
    expect(dialog.getByLabelText('Passwort')).toHaveAccessibleDescription(
      'Gib das Passwort bzw. das Client-Secret ein.',
    )
    expect(onSubmit).not.toHaveBeenCalled()
  })

  it('never shows the stored secret and keeps it when the password stays empty', async () => {
    const dialog = renderDialog(basicTarget)
    const password = dialog.getByLabelText('Passwort')
    expect(password).toHaveValue('')
    expect(password).toHaveAttribute('type', 'password')
    expect(password).toHaveAttribute('autocomplete', 'new-password')
    expect(password).toHaveAccessibleDescription('Gespeichert. Leer lassen, um es zu behalten.')
    expect(dialog.getByLabelText('Benutzername')).toHaveValue('alice')

    await userEvent.click(dialog.getByRole('button', { name: 'Speichern' }))

    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1))
    const request = toDataExportRequest(submittedValues())
    expect(request.auth).toEqual({ kind: 'basic', username: 'alice' })
    expect('password' in request.auth).toBe(false)
  })

  it('asks for the password again when the address moves to another origin', async () => {
    const dialog = renderDialog(basicTarget)
    const url = dialog.getByLabelText('Adresse der SensorThings API')
    await userEvent.clear(url)
    await userEvent.type(url, 'https://frost.example.org/v1.1')

    expect(dialog.getByLabelText('Passwort')).not.toHaveAccessibleDescription(
      'Gespeichert. Leer lassen, um es zu behalten.',
    )
    await userEvent.click(dialog.getByRole('button', { name: 'Speichern' }))

    expect(
      await dialog.findByText('Gib das Passwort bzw. das Client-Secret ein.'),
    ).toBeInTheDocument()
    expect(onSubmit).not.toHaveBeenCalled()
  })

  it('requires the client secret after switching to OAuth2', async () => {
    const dialog = renderDialog(basicTarget)
    await userEvent.click(dialog.getByRole('radio', { name: 'OAuth2' }))
    await userEvent.type(dialog.getByLabelText('Token-Adresse'), 'https://auth.stadt.de/token')
    await userEvent.type(dialog.getByLabelText('Client-ID'), 'green-ecolution')

    const secret = dialog.getByLabelText('Client-Secret')
    expect(secret).toHaveValue('')
    expect(secret).not.toHaveAccessibleDescription('Gespeichert. Leer lassen, um es zu behalten.')

    await userEvent.click(dialog.getByRole('button', { name: 'Speichern' }))

    expect(secret).toHaveAccessibleDescription('Gib das Passwort bzw. das Client-Secret ein.')
    expect(onSubmit).not.toHaveBeenCalled()

    await userEvent.type(secret, 's3cret')
    await userEvent.click(dialog.getByRole('button', { name: 'Speichern' }))

    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1))
    expect(toDataExportRequest(submittedValues()).auth).toEqual({
      kind: 'oauth2_client_credentials',
      tokenUrl: 'https://auth.stadt.de/token',
      clientId: 'green-ecolution',
      clientSecret: 's3cret',
    })
  })

  it('attaches a refused address to the URL field', async () => {
    onSubmit.mockImplementation(() =>
      Promise.reject(
        new ResponseError(
          new Response(JSON.stringify({ error: 'nope', code: 'data_export.url_not_allowed' }), {
            status: 422,
            headers: { 'Content-Type': 'application/json' },
          }),
        ),
      ),
    )
    const dialog = renderDialog()
    const url = dialog.getByLabelText('Adresse der SensorThings API')
    await userEvent.type(url, 'https://frost.elsewhere.de/v1.1')
    await userEvent.click(dialog.getByRole('button', { name: 'Speichern' }))

    await waitFor(() => expect(url).toHaveAccessibleDescription(URL_NOT_ALLOWED))
    expect(onOpenChange).not.toHaveBeenCalledWith(false)
  })

  it('shows any other server error inside the dialog', async () => {
    onSubmit.mockImplementation(() =>
      Promise.reject(
        new ResponseError(
          new Response(JSON.stringify({ error: 'forbidden' }), {
            status: 403,
            headers: { 'Content-Type': 'application/json' },
          }),
        ),
      ),
    )
    const dialog = renderDialog()
    await userEvent.type(
      dialog.getByLabelText('Adresse der SensorThings API'),
      'https://frost.stadt.de/v1.1',
    )
    await userEvent.click(dialog.getByRole('button', { name: 'Speichern' }))

    expect(
      await dialog.findByText('Für diese Aktion fehlt dir die Berechtigung.'),
    ).toBeInTheDocument()
  })
})
