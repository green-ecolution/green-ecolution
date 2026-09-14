import { render, screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { PluginInstallDialog } from './PluginInstallDialog'

const ORGANIZATION = { id: 'org-1', name: 'Stadt Flensburg', memberCount: 0 }

const pluginMatrixOf = () => within(screen.getByRole('group', { name: /rechte des plugins/i }))

const accessMatrixOf = () =>
  within(screen.getByRole('group', { name: /zugriffsrechte für die ansicht/i }))

/**
 * Every case here renders the whole dialog — a Radix Dialog around two 10×4
 * permission matrices, 80 checkboxes — and drives it through user-event. That
 * costs about a second per test on an idle machine, which leaves too little
 * room under the 5s default once the full suite saturates the CPU: tests here
 * timed out in a parallel run while passing on their own.
 */
describe('PluginInstallDialog', { timeout: 20_000 }, () => {
  it('mirrors the plugin permissions into the access permissions until they are edited', async () => {
    const user = userEvent.setup()
    render(<PluginInstallDialog open onSubmit={vi.fn()} onOpenChange={vi.fn()} />)

    await user.click(pluginMatrixOf().getByRole('checkbox', { name: /bäume.*anlegen/i }))

    expect(accessMatrixOf().getByRole('checkbox', { name: /bäume.*anlegen/i })).toBeChecked()
  })

  it('stops mirroring once the access permissions were touched', async () => {
    const user = userEvent.setup()
    render(<PluginInstallDialog open onSubmit={vi.fn()} onOpenChange={vi.fn()} />)

    await user.click(accessMatrixOf().getByRole('checkbox', { name: /bäume.*lesen/i }))
    await user.click(pluginMatrixOf().getByRole('checkbox', { name: /bäume.*anlegen/i }))

    expect(accessMatrixOf().getByRole('checkbox', { name: /bäume.*anlegen/i })).not.toBeChecked()
  })

  it('rejects a non-https external url', async () => {
    const user = userEvent.setup()
    const onSubmit = vi.fn()
    render(<PluginInstallDialog open onSubmit={onSubmit} onOpenChange={vi.fn()} />)

    await user.type(screen.getByLabelText(/slug/i), 'acme')
    await user.type(screen.getByLabelText(/name/i), 'Acme')
    // Keyboard rather than two clicks: opening the Radix listbox by pointer
    // and clicking an option inside its portal is slow enough in jsdom to
    // push this test past the default timeout when the suite runs in parallel.
    screen.getByRole('combobox', { name: /ansicht/i }).focus()
    await user.keyboard('{Enter}{ArrowDown}{Enter}')
    await user.type(screen.getByLabelText(/adresse/i), 'http://plugin.example.com')
    await user.click(screen.getByRole('button', { name: /installieren/i }))

    expect(onSubmit).not.toHaveBeenCalled()
    expect(screen.getByText(/https/i)).toBeVisible()
  })

  it('keeps an access permission the plugin itself does not hold', async () => {
    // The Kataster shape: a plugin that only imports (needs tree:create/update/delete)
    // but whose status view should only require tree:read, so anyone allowed to look at
    // an import log doesn't also need delete rights. The two sets must not be coupled in
    // either direction.
    const user = userEvent.setup()
    const onSubmit = vi.fn()
    render(
      <PluginInstallDialog
        open
        organizations={[ORGANIZATION]}
        onSubmit={onSubmit}
        onOpenChange={vi.fn()}
      />,
    )

    await user.type(screen.getByLabelText(/slug/i), 'kataster')
    await user.type(screen.getByLabelText(/name/i), 'TBZ Baumkataster')
    await user.click(screen.getByRole('combobox', { name: /organisation/i }))
    await user.click(screen.getByRole('option', { name: ORGANIZATION.name }))

    // Grants the access requirement directly, without the plugin ever holding it.
    await user.click(accessMatrixOf().getByRole('checkbox', { name: /bäume.*lesen/i }))

    await user.click(screen.getByRole('button', { name: /installieren/i }))

    expect(onSubmit).toHaveBeenCalledWith(
      expect.objectContaining({
        permissions: [],
        requiredPermissions: ['tree:read'],
      }),
    )
  })
})
