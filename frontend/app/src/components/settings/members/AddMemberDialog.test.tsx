import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { http, HttpResponse, delay } from 'msw'
import { server } from '@/test/mocks/server'
import { fireEvent, render, screen, userEvent, waitFor } from '@/test/utils'
import type { RoleResponse } from '@/api/backendApi'

// Sonner keeps its toasts in a module-wide store, so a toast of one test would
// still be on screen in the next.
const showToast = vi.fn()
vi.mock('@/hooks/createToast', () => ({ default: () => showToast }))

const { default: AddMemberDialog } = await import('./AddMemberDialog')

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
  plan_selectable: true,
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
  await userEvent.type(screen.getByRole('textbox', { name: 'Vorname' }), 'Anna')
  await userEvent.type(screen.getByRole('textbox', { name: 'Nachname' }), 'Schmidt')
  await userEvent.type(
    screen.getByRole('textbox', { name: 'E-Mail-Adresse' }),
    'Anna.Schmidt@tbz.example',
  )
}

const submit = () => userEvent.click(screen.getByRole('button', { name: 'Person anlegen' }))

describe('AddMemberDialog', () => {
  beforeEach(() => {
    vi.stubEnv('VITE_AUTH_BYPASS', 'true')
    showToast.mockClear()
  })
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
    expect(screen.queryByRole('textbox', { name: 'Vorname' })).not.toBeInTheDocument()
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

    expect(screen.getByRole('textbox', { name: 'Vorname' })).toHaveValue('')
    expect(screen.getByRole('textbox', { name: 'E-Mail-Adresse' })).toHaveValue('')
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
      alert.compareDocumentPosition(screen.getByRole('textbox', { name: 'E-Mail-Adresse' })) &
        Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy()
    expect(screen.getByRole('textbox', { name: 'Vorname' })).toHaveValue('Anna')
    expect(screen.getByRole('textbox', { name: 'E-Mail-Adresse' })).toHaveValue(
      'Anna.Schmidt@tbz.example',
    )
  })

  it('reports an uncoded failure with the generic toast and keeps the form', async () => {
    respondWith(500, { error: 'boom' })
    setup()
    await fillForm()
    await submit()

    await waitFor(() =>
      expect(showToast).toHaveBeenCalledWith('Die Person konnte nicht angelegt werden.', 'error'),
    )
    expect(screen.queryByRole('alert')).not.toBeInTheDocument()
    expect(screen.getByRole('textbox', { name: 'Vorname' })).toHaveValue('Anna')
  })

  it('names the cause of a coded failure in the toast', async () => {
    respondWith(422, { error: 'mismatch', code: 'organization_mismatch.role_vs_user' })
    setup()
    await fillForm()
    await submit()

    await waitFor(() =>
      expect(showToast).toHaveBeenCalledWith(
        'Die Rolle gehört zu einer anderen Organisation als die ausgewählte Person.',
        'error',
      ),
    )
    expect(showToast).toHaveBeenCalledTimes(1)
  })

  it('blames the roles for a 403 while at least one is chosen', async () => {
    respondWith(403, { error: 'forbidden' })
    setup()
    await fillForm()
    await userEvent.click(screen.getByRole('combobox', { name: 'Rollen' }))
    await userEvent.click(screen.getByRole('option', { name: 'Planung' }))
    await userEvent.keyboard('{Escape}')
    await submit()

    expect(await screen.findByRole('alert')).toHaveTextContent(
      'Diese Rolle enthält Rechte, die über deine eigenen hinausgehen.',
    )
    expect(showToast).not.toHaveBeenCalled()
    expect(screen.getByRole('textbox', { name: 'Vorname' })).toHaveValue('Anna')
    expect(screen.getByRole('textbox', { name: 'E-Mail-Adresse' })).toHaveValue(
      'Anna.Schmidt@tbz.example',
    )
  })

  it('reports a 403 without a chosen role with the generic toast', async () => {
    respondWith(403, { error: 'forbidden' })
    setup()
    await fillForm()
    await submit()

    await waitFor(() =>
      expect(showToast).toHaveBeenCalledWith('Die Person konnte nicht angelegt werden.', 'error'),
    )
    expect(
      screen.queryByText('Diese Rolle enthält Rechte, die über deine eigenen hinausgehen.'),
    ).not.toBeInTheDocument()
  })

  it('keeps the password on Escape, so only Fertig or the close button leave it', async () => {
    respondCreated()
    const { onOpenChange } = setup()
    await fillForm()
    await submit()
    await screen.findByText(PASSWORD)

    await userEvent.keyboard('{Escape}')

    expect(onOpenChange).not.toHaveBeenCalled()
    expect(screen.getByText(PASSWORD)).toBeInTheDocument()

    await userEvent.click(screen.getByRole('button', { name: /schließen|close/i }))
    expect(onOpenChange).toHaveBeenCalledWith(false)
  })

  it('reports itself busy from submitting until the result is closed', async () => {
    respondCreated()
    const onBusyChange = vi.fn()
    setup({ onBusyChange })
    await fillForm()
    await submit()
    await screen.findByText(PASSWORD)
    expect(onBusyChange).toHaveBeenLastCalledWith(true)

    await userEvent.click(screen.getByRole('button', { name: 'Fertig' }))
    expect(onBusyChange).toHaveBeenLastCalledWith(false)
  })

  it('stops reporting busy when the request fails', async () => {
    respondWith(500, { error: 'boom' })
    const onBusyChange = vi.fn()
    setup({ onBusyChange })
    await fillForm()
    await submit()

    await waitFor(() => expect(onBusyChange).toHaveBeenLastCalledWith(false))
    expect(onBusyChange).toHaveBeenCalledWith(true)
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
    await userEvent.type(screen.getByRole('textbox', { name: 'Benutzername' }), '{Enter}')

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
      expect(screen.getByRole('textbox', { name: 'Vorname' })).toBeInTheDocument()
      expect(held.calls()).toBe(1)

      held.release()
      expect(await screen.findByText(PASSWORD)).toBeInTheDocument()
    })
  })
})
