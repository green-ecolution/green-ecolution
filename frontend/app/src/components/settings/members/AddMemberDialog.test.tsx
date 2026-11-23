import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { http, HttpResponse, delay } from 'msw'
import { server } from '@/test/mocks/server'
import { fireEvent, render, screen, userEvent, waitFor } from '@/test/utils'
import type { RoleResponse } from '@/api/backendApi'
import AddMemberDialog from './AddMemberDialog'

const ROLES: RoleResponse[] = [
  { id: 'r-plan', name: 'Planung', organizationId: 'org-1', permissions: [], isTemplate: false },
  { id: 'r-tpl', name: 'Vorlage', organizationId: null, permissions: [], isTemplate: true },
]

const PASSWORD = 'Xk7mP-q9RtW-3hNvB-8zYcD'

const createdUser = {
  id: 'u-new',
  first_name: 'Anna',
  last_name: 'Schmidt',
  username: 'anna.schmidt',
  email: 'anna.schmidt@tbz.example',
  email_verified: false,
  created_at: '2026-11-23T00:00:00Z',
  driving_licenses: [],
  roles: [],
  status: 'available',
  watering_plan_selectable: true,
}

const bodies: Record<string, unknown>[] = []

const respondWith = (status: number, body: unknown) =>
  server.use(
    http.post('*/v1/users', async ({ request }) => {
      bodies.push((await request.json()) as Record<string, unknown>)
      return HttpResponse.json(body as Record<string, unknown>, { status })
    }),
  )

const respondCreated = () => respondWith(201, { user: createdUser, initial_password: PASSWORD })

const setup = (props: Partial<React.ComponentProps<typeof AddMemberDialog>> = {}) => {
  const onOpenChange = vi.fn()
  const onCreated = vi.fn()
  const view = render(
    <AddMemberDialog
      open
      organizationId="org-1"
      organizationName="TBZ"
      roles={ROLES}
      onOpenChange={onOpenChange}
      onCreated={onCreated}
      {...props}
    />,
  )
  return { onOpenChange, onCreated, ...view }
}

const fillForm = async () => {
  await userEvent.type(screen.getByLabelText('Vorname'), 'Anna')
  await userEvent.type(screen.getByLabelText('Nachname'), 'Schmidt')
  await userEvent.type(screen.getByLabelText('E-Mail-Adresse'), 'Anna.Schmidt@tbz.example')
}

const submit = () => userEvent.click(screen.getByRole('button', { name: 'Person anlegen' }))

describe('AddMemberDialog', () => {
  beforeEach(() => vi.stubEnv('VITE_AUTH_BYPASS', 'true'))
  afterEach(() => vi.unstubAllEnvs())

  it('creates the person in the selected organization with the chosen roles', async () => {
    bodies.length = 0
    respondCreated()
    setup()
    await fillForm()
    await userEvent.click(screen.getByRole('combobox', { name: 'Rollen' }))
    expect(screen.queryByRole('option', { name: 'Vorlage' })).not.toBeInTheDocument()
    await userEvent.click(screen.getByRole('option', { name: 'Planung' }))
    await userEvent.keyboard('{Escape}')
    await submit()

    await screen.findByText('Anna Schmidt ist angelegt')
    expect(bodies).toEqual([
      {
        organization_id: 'org-1',
        first_name: 'Anna',
        last_name: 'Schmidt',
        email: 'Anna.Schmidt@tbz.example',
        username: 'anna.schmidt',
        role_ids: ['r-plan'],
      },
    ])
    expect(bodies[0]).not.toHaveProperty('password')
  })

  it('shows the initial password once after creating', async () => {
    respondCreated()
    setup()
    await fillForm()
    await submit()

    expect(await screen.findByText('Anna Schmidt ist angelegt')).toBeInTheDocument()
    expect(screen.getByText(PASSWORD)).toBeInTheDocument()
    expect(screen.getByText(/bei der ersten Anmeldung ändern/)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Fertig' })).toBeInTheDocument()
    expect(screen.queryByLabelText('Vorname')).not.toBeInTheDocument()
  })

  it('forgets the password when the dialog closes', async () => {
    respondCreated()
    const { onOpenChange, onCreated, rerender } = setup()
    await fillForm()
    await submit()
    await screen.findByText(PASSWORD)

    await userEvent.click(screen.getByRole('button', { name: 'Fertig' }))

    expect(onOpenChange).toHaveBeenCalledWith(false)
    expect(onCreated).toHaveBeenCalledWith(expect.objectContaining({ id: 'u-new' }))
    const dialog = (open: boolean) => (
      <AddMemberDialog
        open={open}
        organizationId="org-1"
        organizationName="TBZ"
        roles={ROLES}
        onOpenChange={onOpenChange}
        onCreated={onCreated}
      />
    )
    rerender(dialog(false))
    expect(document.body).not.toHaveTextContent(PASSWORD)
    rerender(dialog(true))

    expect(screen.getByLabelText('Vorname')).toHaveValue('')
    expect(screen.getByLabelText('E-Mail-Adresse')).toHaveValue('')
    expect(document.body).not.toHaveTextContent(PASSWORD)
  })

  it('does not close on a click outside while the password is shown', async () => {
    respondCreated()
    const { onOpenChange } = setup()
    await fillForm()
    await submit()
    await screen.findByText(PASSWORD)

    fireEvent.pointerDown(document.body)
    fireEvent.click(document.body)

    expect(onOpenChange).not.toHaveBeenCalled()
    expect(screen.getByText(PASSWORD)).toBeInTheDocument()
  })

  it('reports a taken username or e-mail above both fields', async () => {
    respondWith(409, { error: 'taken', code: 'user.identity_taken' })
    setup()
    await fillForm()
    await submit()

    const alert = await screen.findByRole('alert')
    expect(alert).toHaveTextContent(
      'Der Benutzername oder die E-Mail-Adresse ist bereits vergeben.',
    )
    expect(
      alert.compareDocumentPosition(screen.getByLabelText('E-Mail-Adresse')) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy()
    expect(screen.getByLabelText('Vorname')).toHaveValue('Anna')
    expect(screen.getByLabelText('E-Mail-Adresse')).toHaveValue('Anna.Schmidt@tbz.example')
  })

  it('reports any other failure as a toast and keeps the form', async () => {
    respondWith(500, { error: 'boom' })
    setup()
    await fillForm()
    await submit()

    expect(await screen.findByText('Die Person konnte nicht angelegt werden.')).toBeInTheDocument()
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
    expect(screen.getByLabelText('Vorname')).toHaveValue('Anna')
  })

  it('leaves the roles out without the right to read them', async () => {
    bodies.length = 0
    respondCreated()
    setup({ roles: null })
    expect(screen.queryByRole('combobox', { name: 'Rollen' })).not.toBeInTheDocument()
    await fillForm()
    await submit()

    await screen.findByText(PASSWORD)
    expect(bodies[0]).toMatchObject({ role_ids: [] })
  })

  it('does not submit twice', async () => {
    let calls = 0
    server.use(
      http.post('*/v1/users', async () => {
        calls += 1
        await delay('infinite')
        return HttpResponse.json({})
      }),
    )
    setup()
    await fillForm()
    await submit()

    await waitFor(() =>
      expect(screen.getByRole('button', { name: 'Person anlegen' })).toBeDisabled(),
    )
    await userEvent.type(screen.getByLabelText('Benutzername'), '{Enter}')

    expect(calls).toBe(1)
  })

  describe('while the request is pending', () => {
    const holdRequest = () => {
      let release!: () => void
      const gate = new Promise<void>((resolve) => {
        release = resolve
      })
      let calls = 0
      server.use(
        http.post('*/v1/users', async () => {
          calls += 1
          await gate
          return HttpResponse.json(
            { user: createdUser, initial_password: PASSWORD },
            { status: 201 },
          )
        }),
      )
      return { release, calls: () => calls }
    }

    it('cannot be closed by cancel, Escape or an outside click and shows the password afterwards', async () => {
      const held = holdRequest()
      const { onOpenChange } = setup()
      await fillForm()
      await submit()
      await waitFor(() => expect(screen.getByRole('button', { name: 'Abbrechen' })).toBeDisabled())

      await userEvent.click(screen.getByRole('button', { name: 'Abbrechen' }))
      await userEvent.keyboard('{Escape}')
      fireEvent.pointerDown(document.body)
      fireEvent.click(document.body)
      await userEvent.click(screen.getByRole('button', { name: /schließen|close/i }))

      expect(onOpenChange).not.toHaveBeenCalled()
      expect(screen.getByLabelText('Vorname')).toBeInTheDocument()
      expect(held.calls()).toBe(1)

      held.release()
      expect(await screen.findByText(PASSWORD)).toBeInTheDocument()
    })
  })
})
